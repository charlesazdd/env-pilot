use super::theme;
use crate::app::{Language, Message, Page};
use iced::{
    Element, Length,
    widget::{button, column, container, text},
};
/// 按当前页面标记选中的导航项。
pub fn view(current: Page, language: Language) -> Element<'static, Message> {
    let entries = [
        (Page::System, language.text("System", "系统")),
        (Page::User, language.text("User", "用户")),
    ];
    let mut menu = column![
        text("ENVPILOT").size(21),
        text("ENVIRONMENT CONTROL").size(10)
    ]
    .spacing(12);
    for (page, label) in entries {
        let item = button(text(label).size(15))
            .width(Length::Fill)
            .padding(12)
            .on_press(Message::Navigate(page));
        menu = menu.push(
            item.style(move |theme, status| theme::navigation(theme, status, page == current)),
        );
    }
    container(menu)
        .width(240)
        .height(Length::Fill)
        .padding(24)
        .style(theme::glass)
        .into()
}
