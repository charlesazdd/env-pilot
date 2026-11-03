mod app;
mod core;
mod platform;
mod ui;

/// 初始化用户配置和日志后启动桌面窗口。
fn main() -> anyhow::Result<()> {
    let config = core::load_or_init()?;
    let _guard = core::init_logging()?;
    tracing::info!("Starting EnvPilot {}", env!("CARGO_PKG_VERSION"));
    app::run(config).map_err(anyhow::Error::msg)
}
