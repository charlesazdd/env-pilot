use super::theme::SPACING;
use crate::app::{Message, Page};
use iced::{
    Element,
    widget::{column, container, text},
};
/// 显示概览信息或尚未实现的功能页面。
pub fn view(page: Page) -> Element<'static, Message> {
    let (title, description) = match page {
        Page::Overview => ("EnvPilot", "macOS Environment Manager"),
        Page::Environment => (
            "Environment Variables",
            "Manage environment variables in a future release.",
        ),
        Page::Path => ("PATH Manager", "Manage PATH entries in a future release."),
        Page::Profiles => (
            "Profiles",
            "Organize development environments in a future release.",
        ),
        Page::Settings => (
            "Settings",
            "Application preferences will be available here.",
        ),
    };
    let mut content = column![text(title).size(36), text(description).size(18)].spacing(SPACING);
    if page == Page::Overview {
        content = content.push(
            container(
                column![
                    text("Initialization successful").size(20),
                    text(format!("Version {}", env!("CARGO_PKG_VERSION"))),
                    text(format!(
                        "Platform: {} · {}",
                        std::env::consts::OS,
                        std::env::consts::ARCH
                    ))
                ]
                .spacing(12),
            )
            .padding(24)
            .style(container::rounded_box),
        );
    }
    content.into()
}
