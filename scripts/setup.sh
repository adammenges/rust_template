#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TAURI_CLI_VERSION="${TAURI_CLI_VERSION:-2.11.4}"

if ! command -v rustup >/dev/null 2>&1; then
  echo "error: rustup is required. Install it from https://rustup.rs and rerun this script." >&2
  exit 1
fi

rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy

INSTALLED_TAURI_VERSION="$(cargo tauri --version 2>/dev/null | awk '{print $2}' || true)"
if [[ "$INSTALLED_TAURI_VERSION" != "$TAURI_CLI_VERSION" ]]; then
  echo "Installing Tauri CLI $TAURI_CLI_VERSION..."
  cargo install tauri-cli --version "$TAURI_CLI_VERSION" --locked
fi

if [[ "${OSTYPE:-}" == darwin* ]]; then
  if ! xcode-select -p >/dev/null 2>&1; then
    echo "error: Xcode Command Line Tools are required. Run: xcode-select --install" >&2
    exit 1
  fi

  rustup target add aarch64-apple-darwin --toolchain 1.95.0
elif [[ "$(uname -s)" == "Linux" ]]; then
  LINUX_TARGET="$(rustc -vV | sed -n 's/^host: //p')"
  case "$LINUX_TARGET" in
    x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu)
      rustup target add "$LINUX_TARGET" --toolchain 1.95.0
      ;;
    *)
      echo "error: unsupported Linux host target: $LINUX_TARGET" >&2
      exit 1
      ;;
  esac

  if ! command -v pkg-config >/dev/null 2>&1 || ! pkg-config --exists webkit2gtk-4.1; then
    echo "note: install Linux system libraries with ./scripts/install_linux_dependencies.sh"
  fi
else
  echo "error: this template supports macOS and Linux hosts." >&2
  exit 1
fi

echo "Fetching locked dependencies..."
cargo fetch --locked
echo "Setup complete. Run ./scripts/dev.sh to launch the app."
