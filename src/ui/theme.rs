use iced::{
    Background, Border, Color, Theme,
    widget::{button, container, pick_list, text_input},
};

pub const CONTENT_PADDING: u16 = 32;

/// 根据主题亮度选择玻璃面板的半透明填充与边框。
pub fn glass(theme: &Theme) -> container::Style {
    let dark = theme.palette().background.r < 0.5;
    let (surface, edge) = if dark {
        (
            Color::from_rgba(0.12, 0.19, 0.28, 0.82),
            Color::from_rgba(0.54, 0.73, 0.91, 0.11),
        )
    } else {
        (
            Color::from_rgba(1.0, 1.0, 1.0, 0.70),
            Color::from_rgba(0.40, 0.53, 0.69, 0.13),
        )
    };
    container::Style {
        background: Some(Background::Color(surface)),
        border: Border {
            color: edge,
            width: 1.0,
            radius: 22.0.into(),
        },
        ..Default::default()
    }
}

/// 为变量行提供低对比度的半透明层次，避免干扰阅读。
pub fn glass_row(theme: &Theme) -> container::Style {
    let mut style = glass(theme);
    style.border.radius = 14.0.into();
    style
}

/// 使用蓝色标识选中导航，同时让普通导航保持轻盈。
pub fn navigation(theme: &Theme, status: button::Status, selected: bool) -> button::Style {
    let mut style = if selected {
        button::primary(theme, status)
    } else {
        button::text(theme, status)
    };
    style.border.radius = 14.0.into();
    if selected {
        let dark = theme.palette().background.r < 0.5;
        style.background = Some(Background::Color(if dark {
            Color::from_rgb(0.18, 0.35, 0.49)
        } else {
            Color::from_rgb(0.80, 0.88, 0.95)
        }));
        style.text_color = theme.palette().text;
    }
    style
}

/// 为普通操作统一使用柔和蓝色，并区分悬停、按下和不可用状态。
pub fn soft_button(theme: &Theme, status: button::Status) -> button::Style {
    let mut style = button::secondary(theme, status);
    let color = match status {
        button::Status::Active => Color::from_rgb(0.13, 0.38, 0.60),
        button::Status::Hovered => Color::from_rgb(0.18, 0.46, 0.70),
        button::Status::Pressed => Color::from_rgb(0.10, 0.31, 0.50),
        button::Status::Disabled => Color::from_rgb(0.22, 0.29, 0.36),
    };
    style.background = Some(Background::Color(color));
    style.border.radius = 14.0.into();
    style.text_color = Color::WHITE;
    style
}

/// 统一搜索框圆角，并保留 Iced 的焦点提示。
pub fn soft_input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.border.radius = 14.0.into();
    style
}

/// 将语言选择器的形状与其他控件保持一致。
pub fn soft_picker(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let mut style = pick_list::default(theme, status);
    style.border.radius = 14.0.into();
    style
}

/// 用柔和红色表示禁用操作、绿色表示启用操作，并保留悬停反馈。
pub fn toggle_button(theme: &Theme, status: button::Status, enabled: bool) -> button::Style {
    let mut style = button::secondary(theme, status);
    let color = match (enabled, status) {
        (true, button::Status::Hovered) => Color::from_rgb(0.69, 0.27, 0.31),
        (true, _) => Color::from_rgb(0.55, 0.20, 0.25),
        (false, button::Status::Hovered) => Color::from_rgb(0.20, 0.50, 0.37),
        (false, _) => Color::from_rgb(0.14, 0.39, 0.29),
    };
    style.background = Some(Background::Color(color));
    style.text_color = Color::WHITE;
    style.border.radius = 14.0.into();
    style
}
