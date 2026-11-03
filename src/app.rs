use crate::{
    core::{Config, ThemePreference},
    ui,
};
use iced::{Element, Size, Subscription, Task, Theme, window};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Environment,
    Path,
    Profiles,
    Settings,
}
#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Page),
    SystemTheme(iced::theme::Mode),
}
pub struct App {
    pub page: Page,
    preference: ThemePreference,
    system_theme: iced::theme::Mode,
}
/// 创建窗口并订阅系统主题变化。
pub fn run(config: Config) -> iced::Result {
    let size = Size::new(
        config.window.width.max(900) as f32,
        config.window.height.max(600) as f32,
    );
    iced::application(
        move || {
            (
                App {
                    page: Page::Overview,
                    preference: config.theme.clone(),
                    system_theme: iced::theme::Mode::Light,
                },
                iced::system::theme().map(Message::SystemTheme),
            )
        },
        update,
        view,
    )
    .title("EnvPilot")
    .theme(theme)
    .subscription(subscription)
    .window(window::Settings {
        size,
        min_size: Some(Size::new(900.0, 600.0)),
        resizable: true,
        ..Default::default()
    })
    .run()
}
/// 将导航和系统主题事件归并到应用状态。
fn update(app: &mut App, message: Message) {
    match message {
        Message::Navigate(page) => app.page = page,
        Message::SystemTheme(mode) => app.system_theme = mode,
    }
}
fn view(app: &App) -> Element<'_, Message> {
    ui::view(app.page)
}
fn theme(app: &App) -> Theme {
    match app.preference {
        ThemePreference::Light => Theme::Light,
        ThemePreference::Dark => Theme::Dark,
        ThemePreference::System => match app.system_theme {
            iced::theme::Mode::Dark => Theme::Dark,
            _ => Theme::Light,
        },
    }
}
fn subscription(_: &App) -> Subscription<Message> {
    iced::system::theme_changes().map(Message::SystemTheme)
}
