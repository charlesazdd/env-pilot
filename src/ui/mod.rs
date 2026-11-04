mod environment;
mod sidebar;
mod theme;
use crate::app::{App, Language, Message, Page};
use iced::{
    Element, Length,
    widget::{Space, column, container, pick_list, row},
};
/// 组合导航与当前页面，环境变量页面保留实时交互状态。
pub fn view(app: &App) -> Element<'_, Message> {
    let snapshot = if app.page == Page::System {
        &app.system
    } else {
        &app.user
    };
    let content = environment::view(snapshot, app);
    row![
        sidebar::view(app.page, app.language),
        container(
            column![
                row![
                    Space::new().width(Length::Fill),
                    pick_list(Language::ALL, Some(app.language), Message::LanguageChanged)
                        .style(theme::soft_picker)
                        .padding(12)
                ],
                content
            ]
            .spacing(20)
        )
        .padding(theme::CONTENT_PADDING)
        .style(theme::glass)
        .width(Length::Fill)
        .height(Length::Fill)
    ]
    .spacing(16)
    .padding(16)
    .height(Length::Fill)
    .into()
}
