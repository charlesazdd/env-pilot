#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/dist/dev/EnvPilot.app"
cd "$ROOT"
cargo build --locked
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$ROOT/target/debug/envpilot" "$APP/Contents/MacOS/envpilot.new"
mv -f "$APP/Contents/MacOS/envpilot.new" "$APP/Contents/MacOS/envpilot"
cp "$ROOT/packaging/macos/Info.plist" "$APP/Contents/Info.plist"
cp "$ROOT/assets/icons/AppIcon.icns" "$APP/Contents/Resources/AppIcon.icns"
# 本地临时签名使更新后的应用包与可执行文件保持一致。
codesign --force --sign - "$APP"
# 通过应用包启动，macOS 才能从 Info.plist 获取 Dock 图标。
open -n "$APP"
