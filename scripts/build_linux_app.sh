#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname -- "${BASH_SOURCE[0]}")/.."
[[ "$(uname -s)" == Linux ]] || { echo 'Build on native Linux' >&2; exit 1; }
case "$(uname -m)" in
  x86_64) DEB_ARCH=amd64; IMAGE_ARCH=x86_64 ;;
  aarch64) DEB_ARCH=arm64; IMAGE_ARCH=aarch64 ;;
  *) echo 'Linux packaging supports x86-64 and ARM64 only' >&2; exit 1 ;;
esac
[[ "$#" -le 1 && ( "${1:-}" == '' || "${1:-}" == --deb-only ) ]] || { echo 'Usage: build_linux_app.sh [--deb-only]' >&2; exit 1; }
python3 scripts/rename.py --check
APP_PACKAGE="$(python3 -c 'import json; print(json.load(open("app.json"))["package"])')"
APP_EXE="$(python3 -c 'import json; print(json.load(open("app.json"))["executable"])')"
APP_ID="$(python3 -c 'import json; print(json.load(open("app.json"))["identifier"])')"
# Deliberately build release artifacts on Ubuntu 22.04 (glibc 2.35) or your chosen baseline.
if [[ "${1:-}" != --deb-only ]]; then command -v "${APPIMAGETOOL:-appimagetool}" >/dev/null; fi
command -v dpkg-deb >/dev/null || { echo 'Install dpkg-deb before packaging' >&2; exit 1; }
[[ -f assets/icons/AppIcon-1024.png ]] || { echo 'Missing assets/icons/AppIcon-1024.png; provide a 1024px icon before packaging' >&2; exit 1; }
cargo build --locked --release
mkdir -p dist/linux
STAGING="$(mktemp -d "$PWD/dist/.linux.XXXXXX")"
trap 'rm -rf "$STAGING"' EXIT
DEB="$STAGING/deb"
mkdir -p "$DEB/DEBIAN" "$DEB/usr/bin" "$DEB/usr/share/applications" "$DEB/usr/share/icons/hicolor/1024x1024/apps" "$DEB/usr/share/doc/$APP_PACKAGE"
cp "target/release/$APP_EXE" "$DEB/usr/bin/"
python3 scripts/package_metadata.py deb "$DEB/DEBIAN/control" "$DEB_ARCH"
python3 scripts/package_metadata.py desktop "$DEB/usr/share/applications/$APP_ID.desktop"
cp assets/icons/AppIcon-1024.png "$DEB/usr/share/icons/hicolor/1024x1024/apps/$APP_ID.png"
cp LICENSE THIRD_PARTY.md "$DEB/usr/share/doc/$APP_PACKAGE/"
cp -R licenses "$DEB/usr/share/doc/$APP_PACKAGE/"
dpkg-deb --root-owner-group --build "$DEB" "dist/linux/${APP_PACKAGE}_${DEB_ARCH}.deb"
if [[ "${1:-}" == --deb-only ]]; then exit 0; fi
APPDIR="$STAGING/$APP_PACKAGE.AppDir"
mkdir -p "$APPDIR/usr"
cp -R "$DEB/usr/"* "$APPDIR/usr/"
cp "$DEB/usr/share/applications/$APP_ID.desktop" "$APPDIR/"
cp assets/icons/AppIcon-1024.png "$APPDIR/$APP_ID.png"
mkdir -p "$APPDIR/usr/lib"
# Bundle direct and transitive linked libraries except glibc and host graphics drivers.
# ldd is applied only to our own locally-built executable. Hardware Vulkan drivers remain host-owned.
ldd "target/release/$APP_EXE" > "$STAGING/libraries.txt"
if grep -q 'not found' "$STAGING/libraries.txt"; then
  cat "$STAGING/libraries.txt" >&2
  echo 'Missing runtime libraries; fix the host before packaging' >&2; exit 1
fi
while IFS= read -r library; do
  case "$(basename "$library")" in
    libc.so*|libm.so*|libdl.so*|libpthread.so*|librt.so*|ld-linux*|libvulkan.so*|libGL*|libEGL*|libdrm*) continue ;;
  esac
  cp -L "$library" "$APPDIR/usr/lib/"
done < <(awk '/=> \// {print $3}' "$STAGING/libraries.txt")
cat > "$APPDIR/AppRun" <<RUN
#!/bin/sh
APPDIR=\$(CDPATH= cd -- "\$(dirname -- "\$0")" && pwd)
export LD_LIBRARY_PATH="\$APPDIR/usr/lib\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}"
exec "\$APPDIR/usr/bin/$APP_EXE" "\$@"
RUN
chmod +x "$APPDIR/AppRun"
ARCH="$IMAGE_ARCH" "${APPIMAGETOOL:-appimagetool}" "$APPDIR" "dist/linux/${APP_PACKAGE}-${IMAGE_ARCH}.AppImage"
echo 'Verify both packages on the oldest supported host and your X11/Wayland GPU stack before distribution.'
