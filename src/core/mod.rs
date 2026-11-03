use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("无法定位用户配置目录")]
    MissingDirectory,
    #[error("配置文件读写失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("配置解析失败: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("配置序列化失败: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub theme: ThemePreference,
    pub window: WindowSettings,
    pub preferences: Preferences,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowSettings {
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Preferences {
    pub reserved: bool,
}
impl Default for Config {
    /// 创建首次启动使用的稳定默认值。
    fn default() -> Self {
        Self {
            version: 1,
            theme: ThemePreference::System,
            window: WindowSettings {
                width: 1100,
                height: 720,
            },
            preferences: Preferences::default(),
        }
    }
}
/// 读取已有配置；仅在文件不存在时创建默认配置，避免覆盖用户修改。
pub fn load_or_init() -> Result<Config, Error> {
    let dir = dirs::config_dir()
        .ok_or(Error::MissingDirectory)?
        .join("EnvPilot");
    fs::create_dir_all(&dir)?;
    let path = dir.join("config.toml");
    match fs::read_to_string(&path) {
        Ok(data) => Ok(toml::from_str(&data)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let config = Config::default();
            fs::write(path, toml::to_string_pretty(&config)?)?;
            Ok(config)
        }
        Err(e) => Err(e.into()),
    }
}
/// 开发版输出控制台，发布版写入用户日志目录；调用者持有 guard 直至退出。
pub fn init_logging() -> anyhow::Result<Option<tracing_appender::non_blocking::WorkerGuard>> {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
        Ok(None)
    } else {
        let dir: PathBuf = dirs::home_dir()
            .ok_or(Error::MissingDirectory)?
            .join("Library/Logs/EnvPilot");
        fs::create_dir_all(&dir)?;
        let (writer, guard) =
            tracing_appender::non_blocking(tracing_appender::rolling::daily(dir, "envpilot.log"));
        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(writer)
            .init();
        Ok(Some(guard))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_config() {
        let c = Config::default();
        assert_eq!(c.version, 1);
        assert_eq!(c.window.width, 1100);
    }
    #[test]
    fn serialize_config() {
        let s = toml::to_string(&Config::default()).expect("serialize");
        assert!(s.contains("theme = \"system\""));
    }
    #[test]
    fn deserialize_config() {
        let c: Config = toml::from_str("version = 1\ntheme = 'dark'").expect("parse");
        assert_eq!(c.theme, ThemePreference::Dark);
    }
    #[test]
    fn version_information() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
    }
}
