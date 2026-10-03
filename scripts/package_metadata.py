#!/usr/bin/env python3
"""Generate native package metadata from app.json, without shell interpolation."""
import json
import plistlib
import re
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
app = json.loads((ROOT / 'app.json').read_text())
version = re.search(r'^version = "([^"]+)"', (ROOT / 'Cargo.toml').read_text(), re.M)[1]
mode, destination = sys.argv[1], Path(sys.argv[2])
if mode in ('macos', 'verify-macos'):
    info = dict(CFBundleName=app['display_name'], CFBundleDisplayName=app['display_name'],
                CFBundleIdentifier=app['identifier'], CFBundleExecutable=app['executable'],
                CFBundlePackageType='APPL', CFBundleShortVersionString=version,
                CFBundleVersion=version, CFBundleIconFile='icon.icns', LSMinimumSystemVersion='12.0',
                NSHighResolutionCapable=True)
    path = destination / 'Contents/Info.plist'
    if mode == 'macos':
        path.write_bytes(plistlib.dumps(info))
    else:
        assert plistlib.loads(path.read_bytes()) == info, 'Bundle metadata differs from app.json'
        assert (destination / 'Contents/MacOS' / app['executable']).is_file()
        assert (destination / 'Contents/Resources/icon.icns').is_file()
        print('Bundle identity and resources verified')
elif mode == 'desktop':
    destination.write_text('[Desktop Entry]\nType=Application\n'
        f'Name={app["display_name"]}\nExec={app["executable"]}\nIcon={app["identifier"]}\n'
        'Categories=Utility;\nTerminal=false\n')
elif mode == 'deb':
    arch = sys.argv[3]
    destination.write_text(f'Package: {app["package"]}\nVersion: {version}\nArchitecture: {arch}\n'
        f'Maintainer: {app["display_name"]} Developers\nSection: utils\nPriority: optional\n'
        'Depends: libc6 (>= 2.35), fonts-dejavu-core, libfontconfig1, libfreetype6, libxkbcommon0, libxkbcommon-x11-0, '
        'libwayland-client0, libwayland-cursor0, libssl3, libvulkan1, libxcb1, libx11-6\n'
        f'Description: {app["display_name"]} native GPUI desktop application\n')
else:
    raise SystemExit(f'Unknown mode: {mode}')
