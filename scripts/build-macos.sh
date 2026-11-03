#!/bin/bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCH="${1:-$(uname -m)}"
case "$ARCH" in arm64|aarch64) TARGET=aarch64-apple-darwin ;; x86_64) TARGET=x86_64-apple-darwin ;; *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;; esac
cd "$ROOT"
rustup target add "$TARGET"
cargo build --release --locked --target "$TARGET"
