# EnvPilot

完全使用 Rust 构建的轻量级开源 macOS 环境变量管理器。目前提供 Iced 原生窗口和导航占位页面，不读取或修改环境变量。计划支持 Shell 变量、PATH、launchd 会话、配置分组与切换、备份恢复。

系统要求：macOS 12 或更高版本，Apple Silicon 或 Intel。开发需要 Rust Stable 和 Xcode Command Line Tools。在 RustRover 中直接打开目录，运行或调试 `envpilot` Cargo 目标，无需 JavaScript 工具链。

本地运行 `cargo run`，发布构建使用 `cargo build --release`。macOS 打包依次运行 `./scripts/build-macos.sh`、`./scripts/package-macos.sh`、`./scripts/generate-dmg.sh`，可传入 `aarch64` 或 `x86_64` 指定架构。图标为系统临时占位图标；`packaging/macos/Info.plist` 中的 Bundle Identifier 为占位值。

从 GitHub Releases 下载对应 DMG，打开后将 EnvPilot 拖入 Applications。初期构建未签名、未公证，可能被 Gatekeeper 阻止；请参考 Apple 关于打开未识别开发者软件的说明。可通过 `CODESIGN_IDENTITY` 启用签名；公证需要 Apple 开发者凭据，并使用 `xcrun notarytool` 单独提交及装订。请勿提交凭据。

配置保存在用户配置目录，发布日志保存在 `~/Library/Logs/EnvPilot`。采用 MIT 许可证；发布前请替换 LICENSE 中的版权占位符。
