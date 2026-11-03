#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCH="${1:-$(uname -m)}"
case "$ARCH" in arm64|aarch64) ARCH=aarch64 ;; x86_64) ;; *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;; esac
TARGET="$ARCH-apple-darwin"
APP="$ROOT/dist/$ARCH/EnvPilot.app"
BINARY="$ROOT/target/$TARGET/release/envpilot"
[ -f "$BINARY" ] || { echo "Build first: ./scripts/build-macos.sh $ARCH" >&2; exit 1; }
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BINARY" "$APP/Contents/MacOS/envpilot"
cp "$ROOT/packaging/macos/Info.plist" "$APP/Contents/Info.plist"
cp "$ROOT/assets/icons/AppIcon.icns" "$APP/Contents/Resources/AppIcon.icns"
# 显式提供签名身份时启用 hardened runtime；未提供时保留未签名状态。
if [ -n "${CODESIGN_IDENTITY:-}" ]; then codesign --force --options runtime --timestamp --entitlements "$ROOT/packaging/macos/entitlements.plist" --sign "$CODESIGN_IDENTITY" "$APP"; fi
printf '%s\n' "$APP"
