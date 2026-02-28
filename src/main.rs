use iced::alignment::{Horizontal, Vertical};
use iced::keyboard::{self, key};
use iced::widget::{button, column, container, operation, row, scrollable, text, text_input};
use iced::{
    Color, Element, Font, Length, Shadow, Size, Subscription, Task, Theme, application, border,
    theme, window,
};

fn main() -> iced::Result {
    application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .style(App::style)
        .subscription(App::subscription)
        .default_font(Font::MONOSPACE)
        .centered()
        .window(window::Settings {
            size: iced::Size::new(1080.0, 720.0),
            min_size: Some(iced::Size::new(700.0, 520.0)),
            transparent: true,
            blur: true,
            ..window::Settings::default()
        })
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    AppNameChanged(String),
    BundleIdChanged(String),
    BuildApp,
    RunChecks,
    Reset,
    ToggleShortcuts,
    FocusAppName,
    FocusBundleId,
    FocusNext,
    FocusPrevious,
    KeyboardEvent(keyboard::Event),
    WindowResized(Size),
}

#[derive(Debug)]
struct App {
    app_name: String,
    bundle_id: String,
    status: String,
    show_shortcuts: bool,
    window_width: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            app_name: String::from("MyMacApp"),
            bundle_id: String::from("com.example.mymacapp"),
            status: String::from("Ready. Cmd+R runs checks, Cmd+B prints the build command."),
            show_shortcuts: true,
            window_width: 1080.0,
        }
    }
}

const APP_NAME_INPUT_ID: &str = "app-name-input";
const BUNDLE_ID_INPUT_ID: &str = "bundle-id-input";

const ASCII_BANNER_WIDE: &str = r#"  ____           __
 |  _ \ _   _ ___/ /_
 | |_) | | | / __  /
 |  _ <| |_| / /_/ /
 |_| \_\\__,_\\__,_/
"#;

const ASCII_BANNER_COMPACT: &str = r#" __  __  ___
|  \/  |/ _ \
| |\/| | | | |
| |  | | |_| |
|_|  |_|\___/
"#;

impl App {
    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        format!("{} - Terminal UI Shell", self.app_name)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn style(&self, _theme: &Theme) -> theme::Style {
        theme::Style {
            background_color: app_background(),
            text_color: terminal_text(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::listen().map(Message::KeyboardEvent),
            window::resize_events().map(|(_id, size)| Message::WindowResized(size)),
        ])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AppNameChanged(value) => {
                self.app_name = value;
                self.status = format!("APP_NAME set to \"{}\".", self.app_name);
                Task::none()
            }
            Message::BundleIdChanged(value) => {
                self.bundle_id = value;
                self.status = format!("APP_BUNDLE_ID set to \"{}\".", self.bundle_id);
                Task::none()
            }
            Message::BuildApp => {
                self.status = format!(
                    "$ APP_NAME=\"{}\" APP_BUNDLE_ID=\"{}\" ./scripts/build_macos_app.sh",
                    self.app_name.trim(),
                    self.bundle_id.trim()
                );
                Task::none()
            }
            Message::RunChecks => {
                self.status = String::from("$ ./scripts/check.sh");
                Task::none()
            }
            Message::Reset => {
                let window_width = self.window_width;
                *self = Self::default();
                self.window_width = window_width;
                Task::batch([
                    operation::focus(APP_NAME_INPUT_ID),
                    operation::select_all(APP_NAME_INPUT_ID),
                ])
            }
            Message::ToggleShortcuts => {
                self.show_shortcuts = !self.show_shortcuts;
                self.status = if self.show_shortcuts {
                    String::from("Shortcut overlay enabled. Cmd+/ hides it.")
                } else {
                    String::from("Shortcut overlay hidden. Cmd+/ shows it.")
                };
                Task::none()
            }
            Message::FocusAppName => {
                self.status = String::from("Focus: APP_NAME field.");
                Task::batch([
                    operation::focus(APP_NAME_INPUT_ID),
                    operation::select_all(APP_NAME_INPUT_ID),
                ])
            }
            Message::FocusBundleId => {
                self.status = String::from("Focus: APP_BUNDLE_ID field.");
                Task::batch([
                    operation::focus(BUNDLE_ID_INPUT_ID),
                    operation::select_all(BUNDLE_ID_INPUT_ID),
                ])
            }
            Message::FocusNext => {
                self.status = String::from("Focus moved to next control.");
                operation::focus_next()
            }
            Message::FocusPrevious => {
                self.status = String::from("Focus moved to previous control.");
                operation::focus_previous()
            }
            Message::KeyboardEvent(event) => {
                if let Some(shortcut) = shortcut_message(event) {
                    self.update(shortcut)
                } else {
                    Task::none()
                }
            }
            Message::WindowResized(size) => {
                self.window_width = size.width;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let compact = self.window_width < 920.0;
        let content_width = if compact { 720.0 } else { 920.0 };
        let title_size = if compact { 20 } else { 24 };
        let banner = if compact {
            ASCII_BANNER_COMPACT
        } else {
            ASCII_BANNER_WIDE
        };

        let hero = container(
            column![
                text(banner)
                    .size(if compact { 15 } else { 17 })
                    .color(accent_green()),
                text("Rust + iced terminal shell")
                    .size(title_size)
                    .color(bright_terminal_text()),
                text("Keyboard-first controls, centered layout, and command-line style output.")
                    .size(14)
                    .color(dim_terminal_text()),
            ]
            .spacing(8),
        )
        .padding(18)
        .width(Length::Fill)
        .style(terminal_panel_style);

        let config = container(
            column![
                text("[Cmd+1] APP_NAME").size(13).color(dim_terminal_text()),
                text_input("MyMacApp", &self.app_name)
                    .id(APP_NAME_INPUT_ID)
                    .on_input(Message::AppNameChanged)
                    .padding(10)
                    .style(terminal_input_style),
                text("[Cmd+2] APP_BUNDLE_ID")
                    .size(13)
                    .color(dim_terminal_text()),
                text_input("com.example.mymacapp", &self.bundle_id)
                    .id(BUNDLE_ID_INPUT_ID)
                    .on_input(Message::BundleIdChanged)
                    .padding(10)
                    .style(terminal_input_style),
            ]
            .spacing(8),
        )
        .padding(18)
        .width(Length::Fill)
        .style(terminal_panel_style);

        let actions_controls: Element<'_, Message> = if compact {
            column![
                button(text("Run checks [Cmd+R]").color(bright_terminal_text()))
                    .padding(10)
                    .width(Length::Fill)
                    .on_press(Message::RunChecks)
                    .style(action_button_style),
                button(text("Build .app [Cmd+B]").color(bright_terminal_text()))
                    .padding(10)
                    .width(Length::Fill)
                    .on_press(Message::BuildApp)
                    .style(action_button_style),
                button(text("Reset [Cmd+K]").color(bright_terminal_text()))
                    .padding(10)
                    .width(Length::Fill)
                    .on_press(Message::Reset)
                    .style(reset_button_style),
            ]
            .spacing(10)
            .into()
        } else {
            row![
                button(text("Run checks [Cmd+R]").color(bright_terminal_text()))
                    .padding(10)
                    .on_press(Message::RunChecks)
                    .style(action_button_style),
                button(text("Build .app [Cmd+B]").color(bright_terminal_text()))
                    .padding(10)
                    .on_press(Message::BuildApp)
                    .style(action_button_style),
                button(text("Reset [Cmd+K]").color(bright_terminal_text()))
                    .padding(10)
                    .on_press(Message::Reset)
                    .style(reset_button_style),
            ]
            .spacing(10)
            .align_y(Vertical::Center)
            .into()
        };

        let actions = container(
            column![
                text("Actions").size(13).color(dim_terminal_text()),
                actions_controls,
            ]
            .spacing(8),
        )
        .padding(18)
        .width(Length::Fill)
        .style(terminal_panel_style);

        let status = container(
            column![
                text("Output").size(13).color(dim_terminal_text()),
                text(&self.status)
                    .size(14)
                    .color(bright_terminal_text())
                    .width(Length::Fill),
            ]
            .spacing(6),
        )
        .padding(18)
        .width(Length::Fill)
        .style(output_panel_style);

        let command_preview = format!(
            "$ ./scripts/dev.sh\n$ ./scripts/check.sh\n$ APP_NAME=\"{}\" APP_BUNDLE_ID=\"{}\" ./scripts/build_macos_app.sh",
            self.app_name.trim(),
            self.bundle_id.trim()
        );

        let commands = container(
            column![
                text("Command Deck").size(13).color(dim_terminal_text()),
                text(command_preview).size(14).color(accent_green()),
            ]
            .spacing(6),
        )
        .padding(18)
        .width(Length::Fill)
        .style(terminal_panel_style);

        let shortcuts = container(
            column![
                text("Shortcut Overlay (Cmd+/)")
                    .size(13)
                    .color(dim_terminal_text()),
                text("Tab / Shift+Tab  -> cycle focus")
                    .size(14)
                    .color(bright_terminal_text()),
                text("Cmd+1 / Cmd+2    -> jump to APP_NAME / APP_BUNDLE_ID")
                    .size(14)
                    .color(bright_terminal_text()),
                text("Cmd+R / Cmd+B    -> run checks / build command")
                    .size(14)
                    .color(bright_terminal_text()),
                text("Cmd+K            -> reset")
                    .size(14)
                    .color(bright_terminal_text()),
            ]
            .spacing(4),
        )
        .padding(18)
        .width(Length::Fill)
        .style(terminal_panel_style);

        let mut content = column![hero, config, actions, status, commands]
            .spacing(12)
            .max_width(content_width)
            .width(Length::Fill);

        if self.show_shortcuts {
            content = content.push(shortcuts);
        }

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(if compact { 16 } else { 24 })
            .align_x(Horizontal::Center)
            .align_y(Vertical::Top)
            .into()
    }
}

fn shortcut_message(event: keyboard::Event) -> Option<Message> {
    let keyboard::Event::KeyPressed {
        key,
        modifiers,
        repeat,
        ..
    } = event
    else {
        return None;
    };

    if repeat {
        return None;
    }

    if matches!(key, keyboard::Key::Named(key::Named::Tab)) {
        return Some(if modifiers.shift() {
            Message::FocusPrevious
        } else {
            Message::FocusNext
        });
    }

    if !modifiers.command() {
        return None;
    }

    match key.as_ref() {
        keyboard::Key::Character("1") => Some(Message::FocusAppName),
        keyboard::Key::Character("2") => Some(Message::FocusBundleId),
        keyboard::Key::Character("/") => Some(Message::ToggleShortcuts),
        keyboard::Key::Character(value) => match value.to_ascii_lowercase().as_str() {
            "r" => Some(Message::RunChecks),
            "b" => Some(Message::BuildApp),
            "k" => Some(Message::Reset),
            _ => None,
        },
        _ => None,
    }
}

fn app_background() -> Color {
    Color::from_rgba(0.05, 0.07, 0.08, 0.92)
}

fn panel_background() -> Color {
    Color::from_rgba(0.07, 0.10, 0.11, 0.87)
}

fn output_background() -> Color {
    Color::from_rgba(0.04, 0.06, 0.07, 0.92)
}

fn panel_border() -> Color {
    Color::from_rgb(0.20, 0.35, 0.30)
}

fn accent_green() -> Color {
    Color::from_rgb(0.42, 0.90, 0.66)
}

fn bright_terminal_text() -> Color {
    Color::from_rgb(0.87, 0.98, 0.92)
}

fn dim_terminal_text() -> Color {
    Color::from_rgb(0.55, 0.70, 0.63)
}

fn terminal_text() -> Color {
    bright_terminal_text()
}

fn terminal_panel_style(_theme: &Theme) -> container::Style {
    container::Style::default()
        .background(panel_background())
        .border(border::rounded(8).width(1).color(panel_border()))
        .color(terminal_text())
}

fn output_panel_style(_theme: &Theme) -> container::Style {
    container::Style::default()
        .background(output_background())
        .border(border::rounded(8).width(1).color(accent_green()))
        .color(terminal_text())
}

fn action_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Color::from_rgba(0.11, 0.21, 0.18, 0.95).into()),
        text_color: terminal_text(),
        border: border::rounded(7).width(1).color(panel_border()),
        shadow: Shadow::default(),
        snap: false,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgba(0.17, 0.30, 0.24, 0.98).into()),
            border: border::rounded(7).width(1).color(accent_green()),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgba(0.22, 0.36, 0.30, 0.98).into()),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Color::from_rgba(0.11, 0.14, 0.13, 0.85).into()),
            text_color: dim_terminal_text(),
            ..base
        },
    }
}

fn reset_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Color::from_rgba(0.28, 0.12, 0.12, 0.95).into()),
        text_color: bright_terminal_text(),
        border: border::rounded(7)
            .width(1)
            .color(Color::from_rgb(0.90, 0.35, 0.35)),
        shadow: Shadow::default(),
        snap: false,
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgba(0.38, 0.16, 0.16, 0.98).into()),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Color::from_rgba(0.44, 0.18, 0.18, 0.98).into()),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Color::from_rgba(0.18, 0.10, 0.10, 0.85).into()),
            text_color: dim_terminal_text(),
            ..base
        },
    }
}

fn terminal_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Active => panel_border(),
        text_input::Status::Hovered => accent_green(),
        text_input::Status::Focused { .. } => accent_green(),
        text_input::Status::Disabled => dim_terminal_text(),
    };

    text_input::Style {
        background: panel_background().into(),
        border: border::rounded(6).width(1).color(border_color),
        icon: dim_terminal_text(),
        placeholder: dim_terminal_text(),
        value: terminal_text(),
        selection: Color::from_rgba(0.42, 0.90, 0.66, 0.32),
    }
}
