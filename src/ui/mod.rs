mod dashboard;
mod sidebar;
mod theme;
use crate::app::{Message, Page};
use iced::{
    Element, Length,
    widget::{container, row},
};
/// 组合导航和当前页面，使内容随窗口缩放。
pub fn view(page: Page) -> Element<'static, Message> {
    row![
        sidebar::view(page),
        container(dashboard::view(page))
            .padding(theme::CONTENT_PADDING)
            .width(Length::Fill)
            .height(Length::Fill)
    ]
    .height(Length::Fill)
    .into()
}
