#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCH="${1:-$(uname -m)}"
case "$ARCH" in arm64|aarch64) ARCH=aarch64 ;; x86_64) ;; *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;; esac
APP="$ROOT/dist/$ARCH/EnvPilot.app"
[ -d "$APP" ] || { echo "Package the app first" >&2; exit 1; }
VERSION="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$APP/Contents/Info.plist")"
DMG="$ROOT/dist/EnvPilot_${VERSION}_${ARCH}.dmg"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
cp -R "$APP" "$STAGE/EnvPilot.app"
ln -s /Applications "$STAGE/Applications"
rm -f "$DMG"
hdiutil create -volname EnvPilot -srcfolder "$STAGE" -ov -format UDZO "$DMG"
(cd "$ROOT/dist" && shasum -a 256 "$(basename "$DMG")" > "$(basename "$DMG").sha256")
