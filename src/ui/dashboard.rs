use super::theme::{self, SPACING};
use crate::app::{Language, Message, Page};
use iced::{
    Element,
    widget::{column, container, text},
};
/// 显示概览信息或尚未实现的功能页面。
pub fn view(page: Page, language: Language) -> Element<'static, Message> {
    let (title, description) = match page {
        Page::Overview => (
            "EnvPilot",
            language.text("macOS Environment Manager", "macOS 环境管理器"),
        ),
        Page::Environment => (
            language.text("Environment Variables", "环境变量"),
            language.text(
                "Manage environment variables in a future release.",
                "环境变量管理将在后续版本提供。",
            ),
        ),
        Page::Path => (
            language.text("PATH Manager", "PATH 管理"),
            language.text(
                "Manage PATH entries in a future release.",
                "PATH 管理将在后续版本提供。",
            ),
        ),
        Page::Profiles => (
            language.text("Profiles", "配置方案"),
            language.text(
                "Organize development environments in a future release.",
                "开发环境配置将在后续版本提供。",
            ),
        ),
        Page::Settings => (
            language.text("Settings", "设置"),
            language.text(
                "Application preferences will be available here.",
                "应用偏好设置将在此处提供。",
            ),
        ),
    };
    let mut content = column![text(title).size(36), text(description).size(18)].spacing(SPACING);
    if page == Page::Overview {
        content = content.push(
            container(
                column![
                    text(language.text("Initialization successful", "初始化成功")).size(20),
                    text(format!(
                        "{} {}",
                        language.text("Version", "版本"),
                        env!("CARGO_PKG_VERSION")
                    )),
                    text(format!(
                        "{}: {} · {}",
                        language.text("Platform", "平台"),
                        std::env::consts::OS,
                        std::env::consts::ARCH
                    ))
                ]
                .spacing(12),
            )
            .padding(24)
            .style(theme::glass_row),
        );
    }
    content.into()
}
