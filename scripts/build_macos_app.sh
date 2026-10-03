#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname -- "${BASH_SOURCE[0]}")/.."
[[ "$(uname -s)" == Darwin && "$(uname -m)" == arm64 ]] || { echo 'Build on Apple Silicon macOS' >&2; exit 1; }
[[ "$#" == 0 ]] || { echo 'Usage: build_macos_app.sh (no arguments)' >&2; exit 1; }
SIGNING_IDENTITY="${SIGNING_IDENTITY:--}"
if [[ -n "${NOTARY_PROFILE:-}" && "$SIGNING_IDENTITY" == - ]]; then
  echo 'Notarization requires SIGNING_IDENTITY' >&2; exit 1
fi
python3 scripts/rename.py --check
# Identity is validated before use as paths; never evaluate app metadata as shell code.
APP_NAME="$(python3 -c 'import json; print(json.load(open("app.json"))["display_name"])')"
APP_EXE="$(python3 -c 'import json; print(json.load(open("app.json"))["executable"])')"
build_args=(build --locked --release --target aarch64-apple-darwin)
if [[ "${GPUI_PRECOMPILED_SHADERS:-0}" == 1 ]]; then build_args+=(--no-default-features); fi
cargo "${build_args[@]}"
mkdir -p dist
# Build in a fresh staging directory; replace only this app's generated artifact.
STAGING="$(mktemp -d "$PWD/dist/.bundle.XXXXXX")"
trap 'rm -rf "$STAGING"' EXIT
BUNDLE="$STAGING/$APP_NAME.app"
mkdir -p "$BUNDLE/Contents/MacOS" "$BUNDLE/Contents/Resources"
cp "target/aarch64-apple-darwin/release/$APP_EXE" "$BUNDLE/Contents/MacOS/$APP_EXE"
python3 scripts/package_metadata.py macos "$BUNDLE"
[[ -f assets/icons/AppIcon-1024.png ]] || swift scripts/generate_default_icon.swift assets/icons/AppIcon-1024.png
ICONSET="$STAGING/AppIcon.iconset"
mkdir -p "$ICONSET"
for n in 16 32 128 256 512; do
  sips -z "$n" "$n" assets/icons/AppIcon-1024.png --out "$ICONSET/icon_${n}x${n}.png" >/dev/null
  sips -z "$((n*2))" "$((n*2))" assets/icons/AppIcon-1024.png --out "$ICONSET/icon_${n}x${n}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$BUNDLE/Contents/Resources/icon.icns"
cp LICENSE THIRD_PARTY.md "$BUNDLE/Contents/Resources/"
cp -R licenses "$BUNDLE/Contents/Resources/"
if [[ "$SIGNING_IDENTITY" == - ]]; then
  codesign --force --sign - "$BUNDLE"
else
  codesign --force --options runtime --timestamp --sign "$SIGNING_IDENTITY" "$BUNDLE"
fi
codesign --verify --deep --strict "$BUNDLE"
[[ "$(lipo -archs "$BUNDLE/Contents/MacOS/$APP_EXE")" == arm64 ]]
plutil -lint "$BUNDLE/Contents/Info.plist"
python3 scripts/package_metadata.py verify-macos "$BUNDLE"
if [[ -n "${NOTARY_PROFILE:-}" ]]; then
  [[ "$SIGNING_IDENTITY" != - ]] || { echo 'Notarization requires SIGNING_IDENTITY' >&2; exit 1; }
  ditto -c -k --keepParent "$BUNDLE" "$STAGING/app.zip"
  xcrun notarytool submit "$STAGING/app.zip" --keychain-profile "$NOTARY_PROFILE" --wait
  xcrun stapler staple "$BUNDLE"
fi
rm -rf "dist/$APP_NAME.app"
mv "$BUNDLE" "dist/$APP_NAME.app"
echo "Built: dist/$APP_NAME.app"
