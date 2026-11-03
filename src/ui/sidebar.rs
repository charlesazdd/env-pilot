use crate::app::{Message, Page};
use iced::{
    Element, Length,
    widget::{button, column, container, text},
};
/// 按当前页面标记选中的导航项。
pub fn view(current: Page) -> Element<'static, Message> {
    let entries = [
        (Page::Overview, "Overview"),
        (Page::Environment, "Environment Variables"),
        (Page::Path, "PATH Manager"),
        (Page::Profiles, "Profiles"),
        (Page::Settings, "Settings"),
    ];
    let mut menu = column![text("ENVPILOT").size(15)].spacing(12);
    for (page, label) in entries {
        let item = button(text(label).size(15))
            .width(Length::Fill)
            .padding(12)
            .on_press(Message::Navigate(page));
        menu = menu.push(if page == current {
            item.style(button::primary)
        } else {
            item.style(button::text)
        });
    }
    container(menu)
        .width(240)
        .height(Length::Fill)
        .padding(24)
        .style(container::rounded_box)
        .into()
}
