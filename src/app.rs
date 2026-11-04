use crate::{
    core::Config,
    core::environment::{self, Entry, Scope, Snapshot},
    ui,
};
use iced::{Element, Size, Subscription, Task, Theme, window};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Chinese,
    English,
}
impl Language {
    pub const ALL: [Self; 2] = [Self::Chinese, Self::English];
    /// 根据当前语言选择界面文案。
    pub fn text<'a>(self, english: &'a str, chinese: &'a str) -> &'a str {
        match self {
            Self::Chinese => chinese,
            Self::English => english,
        }
    }
}
impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Chinese => "简体中文",
            Self::English => "English",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    System,
    User,
}
#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Page),
    Search(String),
    ToggleValues,
    Refresh,
    LanguageChanged(Language),
    ToggleDeclaration(usize),
    BeginEdit(usize),
    EditValue(String),
    SaveEdit,
    SubmitEdit,
    EditHover(bool),
    PointerReleased,
    BeginAdd,
    AddName(String),
    AddValue(String),
    SaveAdd,
    CancelAdd,
}
pub struct App {
    pub page: Page,
    pub language: Language,
    pub system: Snapshot,
    pub user: Snapshot,
    pub process: Snapshot,
    pub query: String,
    pub show_values: bool,
    pub error: Option<String>,
    pub editing: Option<(Entry, String)>,
    pub adding: bool,
    pub add_name: String,
    pub add_value: String,
    editing_hover: bool,
    skip_release: bool,
}
/// 使用暗色主题创建应用窗口。
pub fn run(config: Config) -> iced::Result {
    let size = Size::new(
        config.window.width.max(900) as f32,
        config.window.height.max(600) as f32,
    );
    iced::application(
        move || App {
            page: Page::System,
            language: Language::default(),
            system: environment::snapshot(Scope::System),
            user: environment::snapshot(Scope::User),
            process: environment::process_snapshot(),
            query: String::new(),
            show_values: false,
            error: None,
            editing: None,
            adding: false,
            add_name: String::new(),
            add_value: String::new(),
            editing_hover: false,
            skip_release: false,
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
/// 将导航、搜索和语言切换事件归并到应用状态。
fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Navigate(page) => {
            if app.editing.is_none() {
                app.page = page;
            }
        }
        Message::Search(query) => {
            if app.editing.is_none() {
                app.query = query;
            }
        }
        Message::ToggleValues => {
            if app.editing.is_none() {
                app.show_values = !app.show_values;
            }
        }
        Message::Refresh if app.editing.is_none() => {
            app.system = environment::snapshot(Scope::System);
            app.user = environment::snapshot(Scope::User);
            app.process = environment::process_snapshot();
        }
        Message::Refresh => {}
        Message::LanguageChanged(language) => app.language = language,
        Message::BeginEdit(index) => {
            if app.page == Page::User
                && app.show_values
                && app.editing.is_none()
                && let Some(entry) = app.user.entries.get(index)
                && entry.toggleable
            {
                app.editing = Some((entry.clone(), entry.value.clone()));
                app.error = None;
                app.editing_hover = true;
                return iced::widget::operation::focus("envpilot-edit");
            }
        }
        Message::EditValue(value) => {
            if let Some((_, draft)) = &mut app.editing {
                *draft = value;
            }
        }
        Message::SaveEdit | Message::SubmitEdit => {
            app.skip_release = matches!(message, Message::SaveEdit);
            if let Some((entry, draft)) = &app.editing {
                match environment::edit(entry, draft) {
                    Ok(()) => {
                        app.editing = None;
                        app.error = None;
                        app.user = environment::snapshot(Scope::User);
                    }
                    Err(error) => app.error = Some(error.to_string()),
                }
            }
        }
        Message::EditHover(hovered) => app.editing_hover = hovered,
        Message::PointerReleased => {
            if app.skip_release {
                app.skip_release = false;
            } else if app.editing.is_some() && !app.editing_hover {
                app.editing = None;
                app.error = None;
            }
        }
        Message::BeginAdd => {
            if app.page == Page::User && app.editing.is_none() {
                app.adding = true;
                app.error = None;
            }
        }
        Message::AddName(value) => app.add_name = value,
        Message::AddValue(value) => app.add_value = value,
        Message::CancelAdd => {
            app.adding = false;
            app.add_name.clear();
            app.add_value.clear();
            app.error = None;
        }
        Message::SaveAdd => {
            if app.page == Page::User && app.adding {
                match environment::add_user_variable(&app.add_name, &app.add_value) {
                    Ok(()) => {
                        app.adding = false;
                        app.add_name.clear();
                        app.add_value.clear();
                        app.error = None;
                        app.query.clear();
                        app.user = environment::snapshot(Scope::User);
                    }
                    Err(error) => app.error = Some(error.to_string()),
                }
            }
        }
        Message::ToggleDeclaration(index) => {
            if app.page == Page::User
                && let Some(entry) = app.user.entries.get(index)
            {
                app.error = environment::toggle(entry)
                    .err()
                    .map(|error| error.to_string());
                app.user = environment::snapshot(Scope::User);
            }
        }
    }
    Task::none()
}
/// 监听鼠标释放，以便点击输入框外时丢弃未保存草稿。
fn subscription(_: &App) -> Subscription<Message> {
    iced::event::listen_with(|event, _, _| match event {
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::PointerReleased)
        }
        _ => None,
    })
}
fn view(app: &App) -> Element<'_, Message> {
    ui::view(app)
}
/// 使用深蓝黑底色和柔和蓝色强调，保持暗色界面的一致性。
fn theme(_: &App) -> Theme {
    let mut palette = iced::theme::Palette::DARK;
    palette.background = iced::color!(0x0b111b);
    palette.text = iced::color!(0xdce4ed);
    palette.primary = iced::color!(0x77b7df);
    Theme::custom("EnvPilot Dark", palette)
}

#[cfg(test)]
mod language_tests {
    use super::Language;
    #[test]
    fn language_choices_have_distinct_labels_and_translations() {
        assert_eq!(Language::Chinese.to_string(), "简体中文");
        assert_eq!(Language::English.to_string(), "English");
        assert_eq!(Language::Chinese.text("Search", "搜索"), "搜索");
        assert_eq!(Language::English.text("Search", "搜索"), "Search");
    }
}
