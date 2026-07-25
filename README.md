# Rust + Tauri Desktop Template

A small, production-minded starting point for native-feeling macOS and Linux apps built with Rust and [Tauri 2](https://v2.tauri.app). The frontend is plain HTML, CSS, and JavaScript: no Node.js, package manager, bundler, or frontend build step.

## What is included

- Rust 2024 backend with typed, validated Tauri IPC and unit tests
- Responsive terminal-inspired UI that remains usable down to a 420 px window
- A macOS title bar overlay and native Linux window decorations
- Platform-aware Command shortcuts on macOS and Ctrl shortcuts on Linux
- Capability-based permissions, a restrictive content security policy, and frozen JavaScript prototypes
- Reproducible Rust, Tauri CLI, and Cargo dependency versions
- Verified Apple Silicon `.app`, Debian, and AppImage packaging
- An Ubuntu 22.04 CI build for portable Linux artifacts
- Dependabot update configuration, agent instructions, and a durable feedback loop

## Requirements

Both platforms require [rustup](https://rustup.rs). The repository pins Rust 1.95.0 and Tauri CLI 2.11.4.

### macOS

- macOS 13 or newer on Apple Silicon
- Xcode Command Line Tools (`xcode-select --install`)

Packaged macOS apps explicitly target `aarch64-apple-darwin`; Intel Macs are not supported.

### Linux

- x86-64 or ARM64 Linux with glibc
- WebKitGTK 4.1, GTK 3, and Tauri's native build dependencies

Install the supported distribution packages, then the pinned Rust tooling:

```bash
./scripts/install_linux_dependencies.sh
./scripts/setup.sh
```

The dependency installer supports apt, dnf, pacman, and zypper. For portable AppImages, build on the oldest Linux base you intend to support. The included CI uses Ubuntu 22.04, one of Tauri's recommended baselines.

## Start here

```bash
./scripts/setup.sh
./scripts/doctor.sh
./scripts/dev.sh
```

The Tauri dev server watches both `ui/` and the Rust crate. You can also open `ui/index.html` directly for a visual-only browser preview; IPC actions are disabled in that mode.

## Check and package

On either supported host:

```bash
./scripts/check.sh
./scripts/build.sh
```

`build.sh` dispatches to the native packaging workflow:

- macOS builds and verifies `dist/Rust Tauri Template.app`.
- Linux builds and verifies `.deb` and `.AppImage` files in `dist/linux/`.

The macOS build is locked to Apple Silicon, applies an ad-hoc signature when needed, and verifies bundle metadata, architecture, icon, and signature. The Linux build uses the native x86-64 or ARM64 target, checks the required development libraries, and verifies every requested package before copying it to `dist/linux/`.

Platform-specific entry points are also available:

```bash
./scripts/build_macos_app.sh
./scripts/build_linux_app.sh
```

Linux packages should be built on Linux, and macOS bundles should be built on macOS. AppImage portability depends on the glibc version of the build host, so use the included Ubuntu 22.04 CI workflow for release artifacts rather than building on an arbitrarily new workstation.

### Customize a build

```bash
APP_NAME="My App" \
APP_BUNDLE_ID="com.example.my-app" \
APP_VERSION="1.2.3" \
./scripts/build.sh
```

Additional build variables:

| Variable | Default | Platform | Purpose |
| --- | --- | --- | --- |
| `APP_NAME` | `productName` in Tauri config | Both | Package and display name |
| `APP_BUNDLE_ID` | `identifier` in Tauri config | Both | Reverse-DNS application identifier |
| `APP_VERSION` | crate version | Both | Package version |
| `ICON_SOURCE` | `assets/icons/AppIcon-1024.png` | Both | Square PNG or SVG icon source |
| `DIST_DIR` | `dist` / `dist/linux` | Both | Final artifact directory |
| `FORCE_ICONS` | `0` | Both | Regenerate every platform icon when set to `1` |
| `LINUX_BUNDLES` | `deb,appimage` | Linux | Comma-separated `deb`, `appimage`, and/or `rpm` selection |

RPM generation is supported with `LINUX_BUNDLES=rpm`, but the package must be built on a Linux host with the required RPM tooling.

## Keyboard map

Use Command on macOS and Ctrl on Linux:

| Shortcut | Action |
| --- | --- |
| <kbd>Mod+1</kbd> / <kbd>Mod+2</kbd> | Focus app name / bundle ID |
| <kbd>Mod+R</kbd> | Preview the repository check command |
| <kbd>Mod+B</kbd> | Validate config and preview the native package command |
| <kbd>Mod+K</kbd> | Reset the demo configuration |
| <kbd>Mod+/</kbd> | Toggle the shortcut panel |

These shortcuts are window-scoped and do not register system-wide hotkeys.

## Project map

```text
ui/                              Static, platform-aware frontend
src-tauri/src/                   Rust commands and app entry point
src-tauri/tauri.conf.json        Shared window, security, and bundle config
src-tauri/tauri.*.conf.json      Host-specific package defaults
src-tauri/capabilities/          Tauri permission grants
assets/icons/                    Cross-platform source app icon
assets/symbols/                  Exported symbols for future screens
scripts/                         Setup, checks, development, and packaging
.github/workflows/linux.yml      Reproducible Linux check and package build
.agents/skills/                  Reusable repository-specific agent workflows
AGENTS.md                        Coding-agent guidance
FEEDBACK.md                      Persistent project-specific corrections
```

Frontend calls use `window.__TAURI__.core.invoke()`, enabled by `withGlobalTauri` in `tauri.conf.json`. Keep backend commands narrow, validate all frontend input again in Rust, and grant only the capabilities a feature requires.

## Common commands

```bash
make                 # Show targets
make setup
make setup-linux     # Install native dependencies on Linux
make doctor
make dev
make check
make icons
make build-app       # Package for the current host
make build-macos
make build-linux
make clean
```

## Rename the template permanently

For a new app, update these together:

1. Package `name`, `version`, and `description` in `src-tauri/Cargo.toml`.
2. The crate path in `src-tauri/src/main.rs` if the package name changes.
3. `productName`, `version`, `identifier`, and window title in `src-tauri/tauri.conf.json`.
4. Default values and copy in `ui/`.
5. `assets/icons/AppIcon-1024.png`, then run `make icons`.

Run `make check` and the native package target after renaming.

## Distribution

Local macOS builds receive an ad-hoc signature suitable for development. Distribution outside your Mac additionally requires an Apple Developer certificate and notarization.

Linux users can install the generated Debian package or run the AppImage directly after making it executable. Sign release artifacts and publish checksums in your release workflow. Keep signing keys and credentials out of the repository.

## Use as a GitHub template

Enable **Settings → General → Template repository**, then choose **Use this template**. Remove any project-specific history or defaults you do not want downstream before publishing.

## License

[MIT](LICENSE)
