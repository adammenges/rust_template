//! Application entity: owns committed state and the workers, independent of window lifetime.
use crate::{
    background::{Event, Outcome, Services},
    domain::{RuntimeState, Settings},
};
use gpui::Context;

pub struct Session {
    pub settings: Settings,
    pub state: RuntimeState,
    pub preview: bool,
    pub settings_error: Option<String>,
    pub state_error: Option<String>,
    pub error: Option<String>,
    pub running: bool,
    pub quitting: bool,
    pub saving: bool,
    pub progress: usize,
    pub status: String,
    pub services: Option<Services>,
}
impl Session {
    pub fn start(&mut self, fail: bool, cx: &mut Context<Self>) {
        if self.preview || self.quitting || self.running {
            return;
        }
        if let Some(services) = &self.services {
            match services.start_job(fail) {
                Ok(()) => {
                    self.running = true;
                    self.progress = 0;
                    self.error = None;
                    self.status = "Calculating…".into();
                }
                Err(e) => self.error = Some(e),
            }
        }
        cx.notify();
    }
    pub fn cancel(&mut self, cx: &mut Context<Self>) {
        if self.preview || self.quitting {
            return;
        }
        if self.running
            && let Some(s) = &self.services
        {
            s.cancel();
            self.status = "Cancelling…".into();
            cx.notify();
        }
    }
    pub fn save(&mut self, settings: Settings, cx: &mut Context<Self>) {
        if self.preview || self.quitting || self.saving || self.settings_error.is_some() {
            return;
        }
        if let Err(e) = settings.validate() {
            self.error = Some(e);
            cx.notify();
            return;
        }
        if let Some(s) = &self.services {
            match s.save_settings(settings) {
                Ok(()) => {
                    self.saving = true;
                    self.error = None;
                }
                Err(e) => self.error = Some(e),
            }
        }
        cx.notify();
    }
    pub fn poll(&mut self, cx: &mut Context<Self>) {
        let Some(s) = &self.services else {
            return;
        };
        let p = s.progress();
        let mut changed = self.running && p != self.progress;
        if self.running {
            self.progress = p;
        }
        while let Some(event) = s.poll() {
            changed = true;
            match event {
                Event::WorkerFailed(e) => {
                    tracing::error!(event = "worker_stopped_unexpectedly", error = %e);
                    self.running = false;
                    self.saving = false;
                    self.status = "Background services unavailable".into();
                    self.error = Some(e);
                }
                Event::Job(outcome) => {
                    self.running = false;
                    match outcome {
                        Outcome::Complete(result) => {
                            self.status = "Calculation complete".into();
                            self.state.last_result = Some(result);
                            if self.state_error.is_none()
                                && let Err(e) = s.save_state(self.state.clone())
                            {
                                self.error = Some(e);
                            }
                        }
                        Outcome::Cancelled => {
                            self.status = "Cancelled • no result committed".into()
                        }
                        Outcome::Failed(e) => {
                            tracing::error!(event = "job_failed", error = %e);
                            self.status = "Calculation failed".into();
                            self.error = Some(e);
                        }
                    }
                }
                Event::SettingsSaved(result) => {
                    self.saving = false;
                    match result {
                        Ok(settings) => {
                            self.settings = settings;
                            // Settings feedback belongs to the Settings pane; preserve job status.
                        }
                        Err(e) => {
                            tracing::error!(event = "settings_save_failed", error = %e);
                            self.error = Some(e);
                        }
                    }
                }
                Event::StateSaved(Err(e)) => {
                    tracing::error!(event = "state_save_failed", error = %e);
                    self.error = Some(e);
                }
                Event::StateSaved(Ok(())) => {}
            }
        }
        if changed {
            cx.notify();
        }
    }
}
