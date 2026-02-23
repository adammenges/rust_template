use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Color, Element, Length, Theme, application, theme, window};

fn main() -> iced::Result {
    application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .style(App::style)
        .window(window::Settings {
            size: iced::Size::new(1080.0, 720.0),
            min_size: Some(iced::Size::new(860.0, 560.0)),
            // Helps the titlebar and content area feel visually continuous on macOS.
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
}

#[derive(Debug)]
struct App {
    app_name: String,
    bundle_id: String,
    status: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            app_name: String::from("MyMacApp"),
            bundle_id: String::from("com.example.mymacapp"),
            status: String::from(
                "Ready. Use the scripts in /scripts to run checks and build a .app bundle.",
            ),
        }
    }
}

impl App {
    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        format!("{} - Rust macOS Template", self.app_name)
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }

    fn style(&self, _theme: &Theme) -> theme::Style {
        theme::Style {
            background_color: Color::from_rgba(0.96, 0.97, 0.99, 0.90),
            text_color: Color::from_rgb(0.10, 0.11, 0.13),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::AppNameChanged(value) => {
                self.app_name = value;
                self.status = String::from(
                    "Updated app name in UI preview. Set APP_NAME when packaging to override bundle name.",
                );
            }
            Message::BundleIdChanged(value) => {
                self.bundle_id = value;
                self.status = String::from(
                    "Updated bundle identifier in UI preview. Set APP_BUNDLE_ID when packaging to apply.",
                );
            }
            Message::BuildApp => {
                self.status = String::from(
                    "Build command: APP_NAME=<name> APP_BUNDLE_ID=<bundle.id> ./scripts/build_macos_app.sh",
                );
            }
            Message::RunChecks => {
                self.status = String::from("Check command: ./scripts/check.sh");
            }
            Message::Reset => {
                *self = Self::default();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let hero = container(
            column![
                text("Rust + iced macOS Template").size(34),
                text("Clean, simple foundation for shipping native-feeling Apple desktop apps.")
                    .size(16),
                text("Use SF Symbols for icons and keep layout calm, spacious, and readable.")
                    .size(16),
            ]
            .spacing(8),
        )
        .padding(24)
        .width(Length::Fill)
        .style(container::rounded_box);

        let config = container(
            column![
                text("App name").size(14),
                text_input("MyMacApp", &self.app_name).on_input(Message::AppNameChanged),
                text("Bundle identifier").size(14),
                text_input("com.example.mymacapp", &self.bundle_id)
                    .on_input(Message::BundleIdChanged),
            ]
            .spacing(10),
        )
        .padding(24)
        .width(Length::Fill)
        .style(container::rounded_box);

        let actions = container(
            row![
                button("Run checks").on_press(Message::RunChecks),
                button("Build .app").on_press(Message::BuildApp),
                button("Reset").on_press(Message::Reset),
            ]
            .spacing(12)
            .align_y(Vertical::Center),
        )
        .padding(24)
        .width(Length::Fill)
        .style(container::rounded_box);

        let status = container(text(&self.status).size(14))
            .padding(16)
            .width(Length::Fill)
            .style(container::bordered_box);

        let hints = container(
            column![
                text("Template commands").size(16),
                text("- ./scripts/dev.sh").size(14),
                text("- ./scripts/check.sh").size(14),
                text("- ./scripts/build_macos_app.sh").size(14),
            ]
            .spacing(8),
        )
        .padding(20)
        .width(Length::Fill)
        .style(container::rounded_box);

        let content = column![hero, config, actions, status, hints]
            .spacing(16)
            .max_width(900)
            .width(Length::Fill);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Top)
            .into()
    }
}
