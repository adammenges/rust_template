#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname -- "${BASH_SOURCE[0]}")/.."
rustc --version
cargo --version
python3 --version
python3 scripts/rename.py --check
case "$(uname -s)" in
  Darwin)
    [[ "$(uname -m)" == arm64 ]] || { echo 'Apple Silicon only'; exit 1; }
    xcode-select -p
    xcrun --show-sdk-path
    command -v swift >/dev/null
    command -v codesign >/dev/null
    echo 'Default GPUI runtime shaders need Command Line Tools; precompiled shaders need full Xcode + Metal tools.'
    ;;
  Linux)
    command -v pkg-config >/dev/null
    pkg-config --exists fontconfig freetype2 xkbcommon wayland-client libssl
    command -v dpkg-deb >/dev/null
    command -v "${APPIMAGETOOL:-appimagetool}" >/dev/null || echo 'Install appimagetool to produce AppImages (or build --deb-only).'
    echo 'Runtime requires X11 or Wayland and a Vulkan-capable graphics stack.'
    ;;
  *) echo 'Supported build hosts: Apple Silicon macOS, Linux x86-64/ARM64'; exit 1 ;;
esac
