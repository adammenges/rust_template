//! Two application-owned workers: cancellable CPU work and serialized persistence.
//! Admission never blocks. Progress is coalesced; terminal replies are bounded and lossless.
use crate::{
    domain::{JobResult, RuntimeState, Settings, calculate_chunk},
    persistence::Store,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

const CHUNKS: usize = 120;
const CHUNK_SIZE: u64 = 100_000;
#[derive(Debug)]
pub enum Outcome {
    Complete(JobResult),
    Cancelled,
    Failed(String),
}
pub enum Event {
    WorkerFailed(String),
    Job(Outcome),
    SettingsSaved(Result<Settings, String>),
    StateSaved(Result<(), String>),
}
enum Write {
    Settings(Settings),
    State(RuntimeState),
}

pub struct Services {
    jobs: Option<SyncSender<bool>>,
    writes: Option<SyncSender<Write>>,
    events: Option<Receiver<Event>>,
    cancelled: Arc<AtomicBool>,
    progress: Arc<AtomicUsize>,
    busy: AtomicBool,
    failure_reported: AtomicBool,
    threads: Vec<JoinHandle<()>>,
}
impl Services {
    pub fn start(store: Store) -> std::io::Result<Self> {
        let (jobs, job_rx) = mpsc::sync_channel(1);
        let (writes, write_rx) = mpsc::sync_channel(2);
        let (events_tx, events) = mpsc::sync_channel(4);
        let cancelled = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicUsize::new(0));
        let job_cancel = cancelled.clone();
        let job_progress = progress.clone();
        let job_events = events_tx.clone();
        let job = thread::Builder::new()
            .name("example-work".into())
            .spawn(move || {
                while let Ok(fail) = job_rx.recv() {
                    let result = calculate(fail, &job_cancel, &job_progress);
                    if job_events.send(Event::Job(result)).is_err() {
                        break;
                    }
                }
            })?;
        let writer = match thread::Builder::new()
            .name("persistence".into())
            .spawn(move || {
                while let Ok(write) = write_rx.recv() {
                    let event = match write {
                        Write::Settings(s) => Event::SettingsSaved(
                            store
                                .save_settings(&s)
                                .map(|()| s)
                                .map_err(|e| format!("{e:#}")),
                        ),
                        Write::State(s) => {
                            Event::StateSaved(store.save_state(&s).map_err(|e| format!("{e:#}")))
                        }
                    };
                    match &event {
                        Event::SettingsSaved(Err(error)) | Event::StateSaved(Err(error)) => {
                            tracing::error!(event = "persistence_write_failed", %error)
                        }
                        _ => {}
                    }
                    let _ = events_tx.send(event); // On quit, keep draining accepted writes after reply disconnect.
                }
            }) {
            Ok(writer) => writer,
            Err(error) => {
                drop(jobs);
                let _ = job.join();
                return Err(error);
            }
        };
        Ok(Self {
            jobs: Some(jobs),
            writes: Some(writes),
            events: Some(events),
            cancelled,
            progress,
            busy: AtomicBool::new(false),
            failure_reported: AtomicBool::new(false),
            threads: vec![job, writer],
        })
    }
    pub fn start_job(&self, fail: bool) -> Result<(), String> {
        self.check_health()?;
        // Hold admission until the terminal event is consumed, so a new job cannot
        // reset cancellation/progress or be confused with an unconsumed result.
        self.busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "A job is already active or awaiting its result".to_owned())?;
        self.cancelled.store(false, Ordering::Release);
        self.progress.store(0, Ordering::Relaxed);
        let result = self
            .jobs
            .as_ref()
            .ok_or("Worker stopped".to_owned())
            .and_then(|jobs| {
                jobs.try_send(fail)
                    .map_err(|e| format!("Work admission failed: {e}"))
            });
        if result.is_err() {
            self.busy.store(false, Ordering::Release);
        }
        result
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    pub fn progress(&self) -> usize {
        self.progress.load(Ordering::Relaxed)
    }
    pub fn poll(&self) -> Option<Event> {
        if let Ok(event) = self.events.as_ref()?.try_recv() {
            if matches!(event, Event::Job(_)) {
                self.busy.store(false, Ordering::Release);
            }
            return Some(event);
        }
        if let Err(error) = self.check_health()
            && !self.failure_reported.swap(true, Ordering::AcqRel)
        {
            self.cancel();
            return Some(Event::WorkerFailed(error));
        }
        None
    }
    fn check_health(&self) -> Result<(), String> {
        if self.threads.iter().any(JoinHandle::is_finished) {
            Err(
                "A background worker stopped unexpectedly. Quit and restart the application."
                    .into(),
            )
        } else {
            Ok(())
        }
    }
    pub fn save_settings(&self, s: Settings) -> Result<(), String> {
        self.write(Write::Settings(s))
    }
    pub fn save_state(&self, s: RuntimeState) -> Result<(), String> {
        self.write(Write::State(s))
    }
    fn write(&self, request: Write) -> Result<(), String> {
        self.check_health()?;
        self.writes
            .as_ref()
            .ok_or("Storage worker stopped")?
            .try_send(request)
            .map_err(|e| format!("Storage admission failed: {e}"))
    }
    /// Call off the UI thread. Cancel computation, drain accepted writes, release threads.
    pub fn shutdown(mut self) {
        self.stop();
    }
    fn stop(&mut self) {
        if self.threads.is_empty() {
            return;
        }
        self.cancel();
        self.jobs.take();
        self.writes.take();
        // Disconnect terminal replies before joining so a full reply queue cannot deadlock quit.
        self.events.take();
        for thread in self.threads.drain(..) {
            if let Err(error) = thread.join() {
                tracing::error!(?error, "worker panicked during shutdown");
            }
        }
        tracing::info!(event = "workers_stopped");
    }
}
impl Drop for Services {
    fn drop(&mut self) {
        self.stop();
    }
}

fn calculate(fail: bool, cancel: &AtomicBool, progress: &AtomicUsize) -> Outcome {
    tracing::info!(event = "job_started", fail);
    let mut checksum = 0_u64;
    for chunk in 0..CHUNKS {
        if cancel.load(Ordering::Acquire) {
            tracing::info!(event = "job_cancelled");
            return Outcome::Cancelled;
        }
        if fail && chunk == CHUNKS / 3 {
            return Outcome::Failed("Example failure at 33%. Retry with Run calculation.".into());
        }
        checksum = checksum.wrapping_add(calculate_chunk(
            chunk as u64 * CHUNK_SIZE,
            (chunk as u64 + 1) * CHUNK_SIZE,
        ));
        progress.store((chunk + 1) * 100 / CHUNKS, Ordering::Relaxed);
        // Pace the demonstration; cancellation is checked every <= 20 ms + one bounded chunk.
        thread::sleep(Duration::from_millis(20));
    }
    if cancel.load(Ordering::Acquire) {
        return Outcome::Cancelled;
    }
    tracing::info!(event = "job_completed", checksum);
    Outcome::Complete(JobResult {
        samples: CHUNKS as u64 * CHUNK_SIZE,
        checksum,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn next_event(s: &Services) -> Event {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(event) = s.poll() {
                return event;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "worker reply timed out"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }
    #[test]
    fn cancellation_and_failure_use_the_real_worker() {
        let d = tempfile::tempdir().unwrap();
        let s = Services::start(Store::new(crate::app_paths::AppPaths {
            root: d.path().into(),
        }))
        .unwrap();
        s.start_job(false).unwrap();
        s.cancel();
        assert!(matches!(next_event(&s), Event::Job(Outcome::Cancelled)));
        s.start_job(true).unwrap();
        assert!(matches!(next_event(&s), Event::Job(Outcome::Failed(_))));
        s.shutdown();
    }
    #[test]
    fn shutdown_drains_accepted_settings_writes() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::new(crate::app_paths::AppPaths {
            root: d.path().into(),
        });
        let s = Services::start(store.clone()).unwrap();
        let settings = Settings {
            compact: true,
            ..Settings::default()
        };
        s.save_settings(settings.clone()).unwrap();
        let state = RuntimeState {
            last_result: Some(JobResult {
                samples: 2,
                checksum: 3,
            }),
        };
        s.save_state(state.clone()).unwrap();
        s.shutdown();
        assert_eq!(store.load_settings().unwrap(), settings);
        assert_eq!(store.load_state().unwrap(), state);
    }
    #[test]
    fn duplicate_admission_cannot_undo_cancellation() {
        let d = tempfile::tempdir().unwrap();
        let s = Services::start(Store::new(crate::app_paths::AppPaths {
            root: d.path().into(),
        }))
        .unwrap();
        s.start_job(false).unwrap();
        s.cancel();
        assert!(s.start_job(false).is_err());
        assert!(s.cancelled.load(Ordering::Acquire));
        assert!(matches!(next_event(&s), Event::Job(Outcome::Cancelled)));
        s.start_job(false).unwrap();
        s.cancel();
        assert!(matches!(next_event(&s), Event::Job(Outcome::Cancelled)));
    }
    #[test]
    fn shutdown_disconnects_full_replies_and_drains_queued_writes() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::new(crate::app_paths::AppPaths {
            root: d.path().into(),
        });
        let s = Services::start(store.clone()).unwrap();
        // Five replies fill the four-slot event queue and block its writer.
        // A sixth accepted write remains queued. Shutdown must drain it too.
        for i in 0..6 {
            s.writes
                .as_ref()
                .unwrap()
                .send(Write::Settings(Settings {
                    workspace_name: format!("Write {i}"),
                    ..Settings::default()
                }))
                .unwrap();
        }
        s.start_job(false).unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            s.shutdown();
            done_tx.send(()).unwrap();
        });
        done_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("shutdown deadlocked");
        handle.join().unwrap();
        assert_eq!(store.load_settings().unwrap().workspace_name, "Write 5");
    }
    #[test]
    fn storage_failure_is_observable_and_worker_recovers() {
        let d = tempfile::tempdir().unwrap();
        let store = Store::new(crate::app_paths::AppPaths {
            root: d.path().into(),
        });
        let original = Settings::default();
        store.save_settings(&original).unwrap();
        let s = Services::start(store.clone()).unwrap();
        // A stale directory cannot be removed as a pending file.
        std::fs::create_dir(d.path().join("settings.json.pending")).unwrap();
        s.save_settings(Settings {
            compact: true,
            ..original.clone()
        })
        .unwrap();
        assert!(matches!(next_event(&s), Event::SettingsSaved(Err(_))));
        assert_eq!(store.load_settings().unwrap(), original);
        std::fs::remove_dir(d.path().join("settings.json.pending")).unwrap();
        s.save_settings(Settings {
            compact: true,
            ..original
        })
        .unwrap();
        assert!(matches!(next_event(&s), Event::SettingsSaved(Ok(_))));
        assert!(store.load_settings().unwrap().compact);
    }
    #[test]
    fn unexpected_worker_exit_is_reported_and_admission_stops() {
        let d = tempfile::tempdir().unwrap();
        let mut s = Services::start(Store::new(crate::app_paths::AppPaths {
            root: d.path().into(),
        }))
        .unwrap();
        s.jobs.take(); // Simulate a dead worker without relying on an injected panic.
        assert!(matches!(next_event(&s), Event::WorkerFailed(_)));
        assert!(s.start_job(false).is_err());
        assert!(s.save_settings(Settings::default()).is_err());
        assert!(s.poll().is_none()); // Report a persistent failure once.
    }
    #[test]
    fn completed_job_matches_domain_calculation() {
        let cancel = AtomicBool::new(false);
        let progress = AtomicUsize::new(0);
        let Outcome::Complete(result) = calculate(false, &cancel, &progress) else {
            panic!("expected completion")
        };
        assert_eq!(
            result.checksum,
            calculate_chunk(0, CHUNKS as u64 * CHUNK_SIZE)
        );
        assert_eq!(progress.load(Ordering::Relaxed), 100);
    }
}
