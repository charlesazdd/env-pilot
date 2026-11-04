# EnvPilot

A lightweight, open-source macOS environment variable manager built entirely with Rust. This initial version provides a native Iced window and navigation placeholders; it does not read or change environment variables.

Planned: shell variables, PATH, launchd sessions, profiles, switching, and backup/restore.

Requires macOS 12 or later, Apple Silicon or Intel. Development requires stable Rust and Xcode Command Line Tools. Open this folder in RustRover and run or debug the `envpilot` Cargo target. No JavaScript toolchain is required.

Run `cargo run`, or use `./scripts/run-macos.sh` on macOS to launch a development app bundle with its Dock icon. Build with `cargo build --release`. To package on macOS, run `./scripts/build-macos.sh`, `./scripts/package-macos.sh`, and `./scripts/generate-dmg.sh`; pass `aarch64` or `x86_64` to select an architecture. The bundle identifier in `packaging/macos/Info.plist` is a placeholder.

Download the appropriate DMG from GitHub Releases, open it and drag EnvPilot to Applications. Initial builds are unsigned and unnotarized and may be blocked by macOS Gatekeeper. See Apple's instructions for opening software from an unidentified developer. Optional signing uses `CODESIGN_IDENTITY`; notarization requires Apple developer credentials and a separate `xcrun notarytool` submission and stapling step. Never commit credentials.

Configuration is stored in the user configuration directory; release logs are in `~/Library/Logs/EnvPilot`. Licensed under MIT; replace the copyright placeholders in LICENSE before distribution.
