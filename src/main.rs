#![cfg_attr(not(any(target_os = "macos", target_os = "linux")), allow(unused))]
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("This template has explicit macOS and Linux adapters only.");
mod app_paths;
mod app_window;
mod background;
mod desktop_ui;
mod diagnostics;
mod domain;
mod identity;
mod persistence;
mod platform;
mod session;
mod text_input;

use anyhow::Result;
use app_window::{AppWindow, Pane};
use clap::Parser;
use domain::{JobResult, RuntimeState, Settings};
use gpui::{App, AppContext, Application, KeyBinding, Menu, MenuItem, Timer, WindowHandle};
use session::Session;
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(version, about = "Reusable native GPUI desktop starter")]
struct Cli {
    /// Deterministic fixtures. No files, workers, or menu-bar entry are created.
    #[arg(long, value_enum)]
    preview: Option<Pane>,
    /// Override per-user storage (for isolated real-app testing).
    #[arg(long, conflicts_with = "preview")]
    data_dir: Option<PathBuf>,
    /// Automatically quit a preview after this many milliseconds.
    #[arg(long, requires = "preview")]
    quit_after_ms: Option<u64>,
    /// Fixture window width, useful for deterministic responsive-layout inspection.
    #[arg(long, requires = "preview", value_parser = clap::value_parser!(u16).range(480..=4096))]
    preview_width: Option<u16>,
    /// Fixture window height.
    #[arg(long, requires = "preview", value_parser = clap::value_parser!(u16).range(480..=2160))]
    preview_height: Option<u16>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let viewport = (
        f32::from(cli.preview_width.unwrap_or(920)),
        f32::from(cli.preview_height.unwrap_or(800)),
    );
    let paths = if cli.preview.is_some() {
        None
    } else {
        Some(app_paths::AppPaths::discover(cli.data_dir)?)
    };
    let _lease = paths.as_ref().map(|p| p.acquire()).transpose()?;
    let logging = diagnostics::init(paths.as_ref())?;
    tracing::info!(
        event = "startup",
        app = identity::DISPLAY_NAME,
        executable = identity::EXECUTABLE,
        preview = cli.preview.is_some()
    );
    let (settings, settings_error, state, state_error, services) = if let Some(paths) = paths {
        // Startup I/O runs before entering GPUI; UI-time reads/writes belong to workers.
        let store = persistence::Store::new(paths);
        let (settings, settings_error) = load_or_report(store.load_settings());
        let (state, state_error) = load_or_report(store.load_state());
        let services = background::Services::start(store)?;
        (settings, settings_error, state, state_error, Some(services))
    } else {
        (
            Settings {
                workspace_name: "Example workspace".into(),
                ..Settings::default()
            },
            None,
            RuntimeState {
                last_result: Some(JobResult {
                    samples: 12_000_000,
                    checksum: 0x4d61_7465_7269_616c,
                }),
            },
            None,
            None,
        )
    };
    let mut session = Session {
        settings,
        settings_error,
        state,
        state_error,
        services,
        preview: cli.preview.is_some(),
        error: None,
        running: false,
        quitting: false,
        saving: false,
        progress: if cli.preview.is_some() { 64 } else { 0 },
        status: if cli.preview.is_some() {
            "Fixture • deterministic progress and result".into()
        } else {
            "Ready".into()
        },
    };
    if cli.preview == Some(Pane::Error) {
        session.error = Some("Example failure at 33%. Retry with Run calculation.".into());
        session.progress = 33;
        session.status = "Calculation failed • fixture".into();
    }
    let session_holder = Rc::new(RefCell::new(None));
    let window: Rc<RefCell<Option<WindowHandle<AppWindow>>>> = Rc::new(RefCell::new(None));
    let reopen_window = window.clone();
    let reopen_session = session_holder.clone();
    let shutdown_thread = Rc::new(RefCell::new(None));
    let run_shutdown_thread = shutdown_thread.clone();
    let run_session_holder = session_holder.clone();
    let application = Application::new();
    application.on_reopen(move |cx| {
        if let Some(session) = reopen_session.borrow().as_ref() {
            open_or_focus(&reopen_window, session, Pane::Home, viewport, cx);
        }
    });
    application.run(move |cx| {
        tracing::info!(event = "application_ready");
        let session = cx.new(|_| session);
        *run_session_holder.borrow_mut() = Some(session.clone());
        let status = Rc::new(RefCell::new(None));
        let native = match platform::NativeApplication::install() {
            Ok(native) => Rc::new(native),
            Err(error) => {
                eprintln!("Native application initialization failed: {error:#}");
                cx.quit();
                return;
            }
        };
        let quit_requested = Rc::new(Cell::new(false));
        let (shutdown_tx, shutdown_rx) = std::sync::mpsc::sync_channel(1);
        let logging = Rc::new(RefCell::new(logging));
        if cli.preview.is_none() {
            match platform::StatusItem::install() {
                Ok(item) => *status.borrow_mut() = Some(item),
                Err(error) => {
                    tracing::error!(%error, "Menu-bar installation failed");
                    session.update(cx, |s, cx| {
                        s.error = Some(format!("Menu bar unavailable: {error}"));
                        cx.notify();
                    });
                }
            }
        }
        let mut bindings = text_input::key_bindings();
        let modifier = if cfg!(target_os = "macos") {
            "cmd"
        } else {
            "ctrl"
        };
        bindings.extend([
            KeyBinding::new(
                &format!("{modifier}-l"),
                app_window::FocusScratchpad,
                Some("Shell"),
            ),
            KeyBinding::new(&format!("{modifier}-1"), app_window::Home, Some("Shell")),
            KeyBinding::new(
                &format!("{modifier}-2"),
                app_window::ShowSettings,
                Some("Shell"),
            ),
            KeyBinding::new(&format!("{modifier}-r"), app_window::RunJob, Some("Shell")),
            KeyBinding::new(
                &format!("{modifier}-."),
                app_window::CancelJob,
                Some("Shell"),
            ),
            KeyBinding::new(
                &format!("{modifier}-shift-r"),
                app_window::FailJob,
                Some("Shell"),
            ),
            KeyBinding::new(
                &format!("{modifier}-s"),
                app_window::SaveSettings,
                Some("Shell"),
            ),
            KeyBinding::new(
                &format!("{modifier}-shift-d"),
                app_window::ToggleCompact,
                Some("Shell"),
            ),
            KeyBinding::new(
                &format!("{modifier}-w"),
                app_window::CloseWindow,
                Some("Shell"),
            ),
        ]);
        bindings.push(KeyBinding::new(
            &format!("{modifier}-q"),
            app_window::Quit,
            None,
        ));
        cx.bind_keys(bindings);
        let action_quit = quit_requested.clone();
        cx.on_action(move |_: &app_window::Quit, _| action_quit.set(true));
        cx.set_menus(vec![Menu {
            name: identity::DISPLAY_NAME.into(),
            items: vec![MenuItem::action("Quit", app_window::Quit)],
        }]);
        open_or_focus(
            &window,
            &session,
            cli.preview.unwrap_or(Pane::Home),
            viewport,
            cx,
        );
        let preview = cli.preview.is_some();
        let close_quit = quit_requested.clone();
        cx.on_window_closed(move |cx| {
            if (preview || cfg!(target_os = "linux")) && cx.windows().is_empty() {
                close_quit.set(true);
            }
        })
        .detach();
        let quit_status = status.clone();
        cx.on_app_quit(move |_| {
            quit_status.borrow_mut().take(); // Main-thread AppKit teardown.
            async {}
        })
        .detach();
        let start = Instant::now();
        cx.spawn(async move |cx| {
            let mut stopping = false;
            let mut timeout_requested = false;
            loop {
                Timer::after(Duration::from_millis(50)).await;
                if !timeout_requested
                    && cli
                        .quit_after_ms
                        .is_some_and(|ms| start.elapsed() >= Duration::from_millis(ms))
                {
                    timeout_requested = true;
                    // Exercise native termination outside a GPUI App borrow.
                    if cfg!(target_os = "macos") {
                        native.request_termination();
                    } else {
                        quit_requested.set(true);
                    }
                }
                let mut should_reply = false;
                if cx
                    .update(|cx| {
                        session.update(cx, |s, cx| s.poll(cx));
                        let actions = status.borrow().as_ref().map_or(0, |s| s.take_actions());
                        if actions & platform::QUIT != 0 {
                            quit_requested.set(true);
                        }
                        if native.take_quit_request() {
                            quit_requested.set(true);
                        }
                        if quit_requested.get() && !stopping {
                            stopping = true;
                            tracing::info!(event = "shutdown_requested");
                            let services = session.update(cx, |s, cx| {
                                s.quitting = true;
                                s.status = "Stopping…".into();
                                cx.notify();
                                s.services.take()
                            });
                            let guard = logging.borrow_mut().take();
                            let reply = shutdown_tx.clone();
                            *run_shutdown_thread.borrow_mut() =
                                Some(std::thread::spawn(move || {
                                    if let Some(s) = services {
                                        s.shutdown();
                                    }
                                    tracing::info!(event = "shutdown_complete");
                                    drop(guard); // Flush diagnostics before AppKit can exit the process.
                                    let _ = reply.send(());
                                }));
                        }
                        if stopping && shutdown_rx.try_recv().is_ok() {
                            native.allow_termination();
                            should_reply = true;
                            cx.quit();
                            return;
                        }
                        if !stopping && actions & platform::OPEN != 0 {
                            open_or_focus(&window, &session, Pane::Home, viewport, cx);
                        }
                    })
                    .is_err()
                {
                    break;
                }
                if should_reply {
                    native.reply_if_pending();
                    break;
                }
            }
        })
        .detach();
    });
    if let Some(thread) = shutdown_thread.borrow_mut().take() {
        let _ = thread.join();
    }
    // Defensive cleanup if the event loop returned without invoking its quit observer.
    if let Some(session) = session_holder.borrow_mut().take() {
        // Normal shutdown already transferred the worker owner above.
        drop(session);
    }
    Ok(())
}
fn load_or_report<T: Default>(result: Result<T>) -> (T, Option<String>) {
    match result {
        Ok(value) => (value, None),
        Err(error) => {
            tracing::error!(%error, "Startup state could not be loaded");
            (T::default(), Some(format!("{error:#}")))
        }
    }
}
fn open_or_focus(
    window: &Rc<RefCell<Option<WindowHandle<AppWindow>>>>,
    session: &gpui::Entity<Session>,
    pane: Pane,
    viewport: (f32, f32),
    cx: &mut App,
) {
    if let Some(handle) = *window.borrow()
        && handle.update(cx, |_, w, _| w.activate_window()).is_ok()
    {
        cx.activate(true);
        return;
    }
    match app_window::open(session.clone(), pane, viewport, cx) {
        Ok(handle) => {
            tracing::info!(
                event = "window_opened",
                width = viewport.0,
                height = viewport.1
            );
            *window.borrow_mut() = Some(handle);
            cx.activate(true);
        }
        Err(error) => {
            tracing::error!(%error,"Could not open window");
            cx.quit();
        }
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;
    #[test]
    fn invalid_flags_are_rejected_before_storage_or_application_startup() {
        for args in [
            vec!["starter", "--preview", "home", "--data-dir", "unused"],
            vec!["starter", "--quit-after-ms", "100"],
            vec!["starter", "--preview-width", "920"],
            vec!["starter", "--preview", "home", "--preview-width", "479"],
            vec!["starter", "--preview", "home", "--preview-height", "2161"],
            vec!["starter", "--preview", "unknown"],
        ] {
            assert!(Cli::try_parse_from(args).is_err());
        }
    }
    #[test]
    fn every_fixture_supports_the_minimum_viewport() {
        for pane in ["home", "settings", "error"] {
            let cli = Cli::try_parse_from([
                "starter",
                "--preview",
                pane,
                "--preview-width",
                "480",
                "--preview-height",
                "480",
                "--quit-after-ms",
                "1",
            ])
            .unwrap();
            assert!(cli.preview.is_some());
            assert!(cli.data_dir.is_none());
        }
    }
}
