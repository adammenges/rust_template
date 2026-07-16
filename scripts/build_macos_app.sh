#!/usr/bin/env bash
set -euo pipefail

if [[ "${OSTYPE:-}" != darwin* ]]; then
  echo "This script builds a macOS .app bundle and must run on macOS."
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required."
  exit 1
fi

read_cargo_field() {
  local key="$1"
  sed -nE "s/^[[:space:]]*${key}[[:space:]]*=[[:space:]]*\"([^\"]+)\".*/\1/p" Cargo.toml | head -n 1
}

prettify_app_name() {
  local raw_name="$1"
  echo "$raw_name" | tr '_-' ' ' | awk '
    {
      for (i = 1; i <= NF; i++) {
        $i = toupper(substr($i, 1, 1)) tolower(substr($i, 2))
      }
      print
    }'
}

PACKAGE_NAME="$(read_cargo_field name)"
PACKAGE_VERSION="$(read_cargo_field version)"

if [[ -z "$PACKAGE_NAME" || -z "$PACKAGE_VERSION" ]]; then
  echo "Could not read package metadata from Cargo.toml."
  exit 1
fi

APP_EXECUTABLE="${APP_EXECUTABLE:-$PACKAGE_NAME}"
DEFAULT_APP_NAME="$(prettify_app_name "$PACKAGE_NAME")"
APP_NAME="${APP_NAME:-$DEFAULT_APP_NAME}"
APP_BUNDLE_ID="${APP_BUNDLE_ID:-com.example.${PACKAGE_NAME//_/-}}"
APP_VERSION="${APP_VERSION:-$PACKAGE_VERSION}"
ICON_SOURCE="${ICON_SOURCE:-assets/icons/AppIcon-1024.png}"
ICON_ICNS="${ICON_ICNS:-assets/icons/AppIcon.icns}"
DIST_DIR="${DIST_DIR:-dist}"

cargo tauri build

mkdir -p "$DIST_DIR"

APP_BUNDLE="$(find src-tauri/target/release/bundle/macos -name "*.app" -maxdepth 1 2>/dev/null | head -n 1)"
if [[ -n "$APP_BUNDLE" ]]; then
  rm -rf "$DIST_DIR/$(basename "$APP_BUNDLE")"
  cp -R "$APP_BUNDLE" "$DIST_DIR/"
  echo "Built macOS app bundle at $DIST_DIR/$(basename "$APP_BUNDLE")"
else
  echo "Build completed but .app bundle not found. Check src-tauri/target/release/bundle/"
fi
