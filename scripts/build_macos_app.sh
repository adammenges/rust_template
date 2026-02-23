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

PACKAGE_NAME="$(read_cargo_field name)"
PACKAGE_VERSION="$(read_cargo_field version)"

if [[ -z "$PACKAGE_NAME" || -z "$PACKAGE_VERSION" ]]; then
  echo "Could not read package metadata from Cargo.toml."
  exit 1
fi

APP_EXECUTABLE="${APP_EXECUTABLE:-$PACKAGE_NAME}"
APP_NAME="${APP_NAME:-$PACKAGE_NAME}"
APP_BUNDLE_ID="${APP_BUNDLE_ID:-com.example.${PACKAGE_NAME//_/-}}"
APP_VERSION="${APP_VERSION:-$PACKAGE_VERSION}"
ICON_SOURCE="${ICON_SOURCE:-assets/icons/AppIcon-1024.png}"
ICON_ICNS="${ICON_ICNS:-assets/icons/AppIcon.icns}"
DIST_DIR="${DIST_DIR:-dist}"
BUILD_MODE="${BUILD_MODE:-release}"
UNIVERSAL="${UNIVERSAL:-0}"

if [[ "$BUILD_MODE" == "release" ]]; then
  CARGO_FLAGS=(--release)
  PROFILE_DIR="release"
else
  CARGO_FLAGS=()
  PROFILE_DIR="debug"
fi

APP_DIR="$DIST_DIR/$APP_NAME.app"
MACOS_DIR="$APP_DIR/Contents/MacOS"
RESOURCES_DIR="$APP_DIR/Contents/Resources"
PLIST_PATH="$APP_DIR/Contents/Info.plist"
OUTPUT_BINARY="$MACOS_DIR/$APP_EXECUTABLE"

resolve_binary() {
  local base="$1"
  if [[ -f "$base/$APP_EXECUTABLE" ]]; then
    echo "$base/$APP_EXECUTABLE"
    return
  fi

  local alt_name="${APP_EXECUTABLE//-/_}"
  if [[ -f "$base/$alt_name" ]]; then
    echo "$base/$alt_name"
    return
  fi

  echo ""
}

rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR"

if [[ "$UNIVERSAL" == "1" ]]; then
  rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null

  cargo build "${CARGO_FLAGS[@]}" --target aarch64-apple-darwin
  cargo build "${CARGO_FLAGS[@]}" --target x86_64-apple-darwin

  ARM_BASE="target/aarch64-apple-darwin/$PROFILE_DIR"
  X64_BASE="target/x86_64-apple-darwin/$PROFILE_DIR"
  ARM_BIN="$(resolve_binary "$ARM_BASE")"
  X64_BIN="$(resolve_binary "$X64_BASE")"

  if [[ -z "$ARM_BIN" || -z "$X64_BIN" ]]; then
    echo "Failed to find one or more target binaries for universal build."
    exit 1
  fi

  lipo -create "$ARM_BIN" "$X64_BIN" -output "$OUTPUT_BINARY"
else
  cargo build "${CARGO_FLAGS[@]}"

  LOCAL_BASE="target/$PROFILE_DIR"
  LOCAL_BIN="$(resolve_binary "$LOCAL_BASE")"

  if [[ -z "$LOCAL_BIN" ]]; then
    echo "Binary not found under $LOCAL_BASE. Set APP_EXECUTABLE if your binary name differs."
    exit 1
  fi

  cp "$LOCAL_BIN" "$OUTPUT_BINARY"
fi

chmod +x "$OUTPUT_BINARY"

if [[ ! -f "$ICON_SOURCE" ]]; then
  echo "No icon found at $ICON_SOURCE. Generating a fallback icon."
  mkdir -p "$(dirname "$ICON_SOURCE")"

  if command -v swift >/dev/null 2>&1; then
    if swift scripts/generate_default_icon.swift "$ICON_SOURCE" >/dev/null 2>&1; then
      echo "Generated fallback icon with Swift."
    else
      echo "Swift icon generation failed. Falling back to macOS generic app icon."
    fi
  fi

  if [[ ! -f "$ICON_SOURCE" ]]; then
    GENERIC_ICON_ICNS="/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns"
    if [[ -f "$GENERIC_ICON_ICNS" ]]; then
      TMP_BASE="$(mktemp -t generic-app-icon)"
      TMP_PNG="${TMP_BASE}.png"
      sips -s format png "$GENERIC_ICON_ICNS" --out "$TMP_PNG" >/dev/null
      sips -z 1024 1024 "$TMP_PNG" --out "$ICON_SOURCE" >/dev/null
      rm -f "$TMP_PNG" "$TMP_BASE"
      echo "Generated fallback icon from GenericApplicationIcon.icns."
    fi
  fi

  if [[ ! -f "$ICON_SOURCE" ]]; then
    echo "Failed to generate a fallback icon. Add a 1024x1024 PNG at $ICON_SOURCE and rerun."
    exit 1
  fi
fi

./scripts/make_icns.sh "$ICON_SOURCE" "$ICON_ICNS"
cp "$ICON_ICNS" "$RESOURCES_DIR/AppIcon.icns"

cat > "$PLIST_PATH" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>$APP_NAME</string>
  <key>CFBundleExecutable</key>
  <string>$APP_EXECUTABLE</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundleIdentifier</key>
  <string>$APP_BUNDLE_ID</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>$APP_NAME</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>$APP_VERSION</string>
  <key>CFBundleVersion</key>
  <string>$APP_VERSION</string>
  <key>LSApplicationCategoryType</key>
  <string>public.app-category.developer-tools</string>
  <key>LSMinimumSystemVersion</key>
  <string>13.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

if command -v codesign >/dev/null 2>&1; then
  codesign --force --deep --sign - "$APP_DIR" >/dev/null 2>&1 || true
fi

echo "Built macOS app bundle at $APP_DIR"
