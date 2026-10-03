use crate::{
    desktop_ui as ui, domain::Settings, identity, session::Session, text_input::TextInput,
};
use gpui::{
    App, Bounds, Context, Entity, FocusHandle, Focusable, Render, Subscription, TitlebarOptions,
    Window, WindowBounds, WindowHandle, WindowOptions, actions, div, prelude::*, px, rgb, size,
};

actions!(
    shell,
    [
        Home,
        ShowSettings,
        RunJob,
        CancelJob,
        FailJob,
        SaveSettings,
        ToggleCompact,
        FocusScratchpad,
        Quit,
        CloseWindow
    ]
);
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Pane {
    Home,
    Settings,
    Error,
}

pub struct AppWindow {
    session: Entity<Session>,
    pane: Pane,
    focus: FocusHandle,
    name: Entity<TextInput>,
    notes: Entity<TextInput>,
    compact: bool,
    _subscription: Subscription,
    _input_subscription: Subscription,
    scroll: gpui::ScrollHandle,
}
impl AppWindow {
    fn new(
        session: Entity<Session>,
        pane: Pane,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let settings = session.read(cx).settings.clone();
        let name = cx.new(|cx| TextInput::new(cx, "Workspace name", settings.workspace_name));
        let notes = cx.new(|cx| {
            TextInput::multiline(
                cx,
                "Try typing, selecting, or pasting text…",
                "Rust owns the interface.\nThis scratchpad stays in memory.",
            )
        });
        let focus = cx.focus_handle();
        if pane == Pane::Settings {
            window.focus(&name.focus_handle(cx));
        } else {
            window.focus(&focus);
        }
        let subscription = cx.observe(&session, |_, _, cx| cx.notify());
        let input_subscription = cx.observe(&name, |_, _, cx| cx.notify());
        Self {
            session,
            pane,
            focus,
            name,
            notes,
            compact: settings.compact,
            _subscription: subscription,
            _input_subscription: input_subscription,
            scroll: gpui::ScrollHandle::default(),
        }
    }
    fn home(&mut self, _: &Home, window: &mut Window, cx: &mut Context<Self>) {
        self.pane = Pane::Home;
        self.scroll.set_offset(gpui::point(px(0.), px(0.)));
        window.focus(&self.focus);
        cx.notify();
    }
    fn settings(&mut self, _: &ShowSettings, window: &mut Window, cx: &mut Context<Self>) {
        self.pane = Pane::Settings;
        self.scroll.set_offset(gpui::point(px(0.), px(0.)));
        window.focus(&self.name.focus_handle(cx));
        cx.notify();
    }
    fn focus_scratchpad(
        &mut self,
        _: &FocusScratchpad,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pane = Pane::Home;
        self.scroll.scroll_to_bottom();
        window.focus(&self.notes.focus_handle(cx));
        cx.notify();
    }
    fn run(&mut self, _: &RunJob, _: &mut Window, cx: &mut Context<Self>) {
        self.session.update(cx, |s, cx| s.start(false, cx));
    }
    fn fail(&mut self, _: &FailJob, _: &mut Window, cx: &mut Context<Self>) {
        self.session.update(cx, |s, cx| s.start(true, cx));
    }
    fn cancel(&mut self, _: &CancelJob, _: &mut Window, cx: &mut Context<Self>) {
        self.session.update(cx, |s, cx| s.cancel(cx));
    }
    fn toggle(&mut self, _: &ToggleCompact, _: &mut Window, cx: &mut Context<Self>) {
        if !self.session.read(cx).preview && !self.session.read(cx).quitting {
            self.compact = !self.compact;
            cx.notify();
        }
    }
    fn save(&mut self, _: &SaveSettings, _: &mut Window, cx: &mut Context<Self>) {
        if !self.has_unsaved_changes(cx) {
            return;
        }
        let settings = Settings {
            workspace_name: self.name.read(cx).text().trim().to_owned(),
            compact: self.compact,
            ..Settings::default()
        };
        self.session.update(cx, |s, cx| s.save(settings, cx));
    }
    fn render_home(&self, cx: &Context<Self>) -> impl IntoElement {
        let s = self.session.read(cx);
        let disabled = s.preview || s.quitting || s.running;
        let result = s
            .state
            .last_result
            .as_ref()
            .map(|r| format!("{} samples · checksum {:016x}", r.samples, r.checksum))
            .unwrap_or_else(|| "No completed result yet".into());
        ui::pane(
            "A small native workspace. Everything here is built in Rust.",
            s.settings.compact,
        )
        .child(
            ui::card(s.settings.compact)
                .child(ui::section_label("WORKSPACE"))
                .child(
                    div()
                        .text_size(px(20.))
                        .child(s.settings.workspace_name.clone()),
                )
                .child(ui::hint(
                    "Rename this workspace in Settings. Choose a density that feels right.",
                )),
        )
        .child(
            ui::card(s.settings.compact)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child("Background calculation")
                        .child(ui::hint(format!("{:03}%", s.progress))),
                )
                .child(ui::hint(
                    "Calculate a checksum over 12 million samples on a dedicated worker.",
                ))
                .child(
                    div()
                        .h(px(5.))
                        .w_full()
                        .rounded_md()
                        .bg(rgb(ui::LINE))
                        .child(
                            div()
                                .h_full()
                                .w(gpui::relative(s.progress as f32 / 100.))
                                .bg(rgb(ui::ACCENT)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .child(
                            ui::button("run", "Run calculation", "⌘/Ctrl R", true, disabled)
                                .on_click(
                                    cx.listener(|this, _, window, cx| {
                                        this.run(&RunJob, window, cx)
                                    }),
                                ),
                        )
                        .child(
                            ui::button(
                                "cancel",
                                "Cancel",
                                "⌘/Ctrl .",
                                false,
                                s.preview || s.quitting || !s.running,
                            )
                            .on_click(cx.listener(
                                |this, _, window, cx| this.cancel(&CancelJob, window, cx),
                            )),
                        )
                        .child(
                            ui::button("fail", "Try failure", "⌘/Ctrl ⇧R", false, disabled)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.fail(&FailJob, window, cx)
                                })),
                        ),
                )
                .child(ui::hint(s.status.clone()))
                .child(ui::hint(result)),
        )
        .child(
            ui::card(s.settings.compact)
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child("Scratchpad")
                        .child(ui::hint(if cfg!(target_os = "macos") {
                            "⌘ L"
                        } else {
                            "Ctrl L"
                        })),
                )
                .child(ui::hint(
                    "Unicode editing, selection, clipboard, undo, and native input methods.",
                ))
                .child(self.notes.clone()),
        )
    }
    fn has_unsaved_changes(&self, cx: &Context<Self>) -> bool {
        let settings = &self.session.read(cx).settings;
        self.name.read(cx).text().trim() != settings.workspace_name
            || self.compact != settings.compact
    }
    fn render_settings(&self, cx: &Context<Self>) -> impl IntoElement {
        let s = self.session.read(cx);
        ui::pane(
            "Personalize your workspace. Changes apply after saving.",
            s.settings.compact,
        )
        .child(ui::section_label("GENERAL"))
        .child(
            ui::card(s.settings.compact)
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child("Workspace name"),
                )
                .child(ui::hint("1–64 characters. Displayed on Home."))
                .child(self.name.clone())
                .child(
                    div()
                        .mt_2()
                        .pt_3()
                        .border_t_1()
                        .border_color(rgb(ui::LINE))
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_4()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_size(px(13.))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .child("Compact spacing"),
                                )
                                .child(ui::hint(
                                    "Reduce spacing across every pane. ⌘/Ctrl ⇧D".replace(
                                        "⌘/Ctrl",
                                        if cfg!(target_os = "macos") {
                                            "⌘"
                                        } else {
                                            "Ctrl"
                                        },
                                    ),
                                )),
                        )
                        .child(
                            ui::toggle("density", self.compact, s.preview || s.quitting).on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.toggle(&ToggleCompact, window, cx)
                                }),
                            ),
                        ),
                ),
        )
        .child(ui::hint(if s.preview {
            "Preview edits are temporary"
        } else if s.saving {
            "Saving changes…"
        } else if s.settings_error.is_some() {
            "Saving is blocked until the settings file is repaired and the app restarted."
        } else if self.has_unsaved_changes(cx) {
            "Unsaved changes • Save to apply"
        } else {
            "All changes saved"
        }))
        .child(ui::hint(if s.preview {
            "Fixture preview • edits stay in memory; saving and jobs are disabled."
        } else {
            "Preferences are stored locally for your account."
        }))
    }
}
impl Focusable for AppWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Render for AppWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let s = self.session.read(cx);
        // Show independent failures together: a corrupt settings file must not hide a job error.
        let mut errors = Vec::new();
        if let Some(e) = &s.settings_error {
            errors.push(format!("Settings could not be loaded: {e}. File preserved. Repair or move it, then restart."));
        }
        if let Some(e) = &s.state_error {
            errors.push(format!("Runtime state could not be loaded: {e}. File preserved; new results stay in memory until restart."));
        }
        errors.extend(s.error.clone());
        let preview = s.preview;
        let narrow = window.viewport_size().width < px(700.);
        let sidebar_width = ui::sidebar_width(f32::from(window.viewport_size().width));
        let action = if self.pane == Pane::Settings {
            ui::button(
                "save",
                if s.saving { "Saving…" } else { "Save" },
                "⌘/Ctrl S",
                true,
                preview
                    || s.quitting
                    || s.saving
                    || s.settings_error.is_some()
                    || !self.has_unsaved_changes(cx),
            )
            .on_click(cx.listener(|this, _, window, cx| this.save(&SaveSettings, window, cx)))
            .into_any_element()
        } else {
            ui::hint(if preview {
                "Fixture preview"
            } else {
                "Local workspace"
            })
            .into_any_element()
        };
        div()
            .key_context("Shell")
            .track_focus(&self.focus)
            .size_full()
            .flex()
            .overflow_hidden()
            .bg(rgb(ui::CANVAS))
            .text_color(rgb(ui::TEXT))
            .font_family(if cfg!(target_os = "macos") {
                ".SystemUIFont"
            } else {
                "DejaVu Sans"
            })
            .text_size(px(13.))
            .on_action(cx.listener(Self::focus_scratchpad))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::settings))
            .on_action(cx.listener(Self::run))
            .on_action(cx.listener(Self::fail))
            .on_action(cx.listener(Self::cancel))
            .on_action(cx.listener(Self::save))
            .on_action(cx.listener(Self::toggle))
            .on_action(|_: &CloseWindow, window, _| window.remove_window())
            .child(
                div()
                    .w(px(sidebar_width))
                    .h_full()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .px(px(if narrow { 8. } else { 14. }))
                    .pt(px(52.))
                    .pb_4()
                    .bg(rgb(ui::SIDEBAR))
                    .border_r_1()
                    .border_color(rgb(ui::LINE))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                ui::navigation(
                                    "home",
                                    "Home",
                                    "house",
                                    self.pane != Pane::Settings,
                                )
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.home(&Home, window, cx)),
                                ),
                            )
                            .child(
                                ui::navigation(
                                    "settings",
                                    "Settings",
                                    "slider.horizontal.3",
                                    self.pane == Pane::Settings,
                                )
                                .on_click(cx.listener(
                                    |this, _, window, cx| this.settings(&ShowSettings, window, cx),
                                )),
                            ),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .px_2()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(ui::hint(identity::DISPLAY_NAME))
                            .child(ui::hint(format!("Version {}", env!("CARGO_PKG_VERSION"))))
                            .child(ui::hint(if cfg!(target_os = "macos") {
                                "⌘1 / ⌘2 · ⌘Q Quit"
                            } else {
                                "Ctrl 1 / 2 · Ctrl Q"
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(ui::pane_header(
                        if self.pane == Pane::Settings {
                            "Settings"
                        } else {
                            "Home"
                        },
                        action,
                        narrow,
                    ))
                    .child(
                        div()
                            .id("pane-scroll")
                            .track_scroll(&self.scroll)
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .px(px(if narrow { 16. } else { 32. }))
                            .py_5()
                            .child(
                                div()
                                    .w_full()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .gap_4()
                                    .children(errors.into_iter().map(|e| {
                                        div()
                                            .w_full()
                                            .max_w(px(ui::CONTENT_WIDTH))
                                            .p_3()
                                            .rounded(px(6.))
                                            .border_1()
                                            .border_color(rgb(0x55413f))
                                            .bg(rgb(0x221a19))
                                            .text_size(px(12.))
                                            .text_color(rgb(ui::NEGATIVE))
                                            .child(e)
                                    }))
                                    .child(if self.pane == Pane::Settings {
                                        self.render_settings(cx).into_any_element()
                                    } else {
                                        self.render_home(cx).into_any_element()
                                    }),
                            ),
                    ),
            )
    }
}

pub fn open(
    session: Entity<Session>,
    pane: Pane,
    viewport: (f32, f32),
    cx: &mut App,
) -> gpui::Result<WindowHandle<AppWindow>> {
    let bounds = Bounds::centered(None, size(px(viewport.0), px(viewport.1)), cx);
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(480.), px(480.))),
            titlebar: Some(TitlebarOptions {
                title: Some(identity::DISPLAY_NAME.into()),
                appears_transparent: true,
                ..Default::default()
            }),
            app_id: Some(identity::APP_ID.into()),
            ..Default::default()
        },
        move |window, cx| cx.new(|cx| AppWindow::new(session, pane, window, cx)),
    )
}
