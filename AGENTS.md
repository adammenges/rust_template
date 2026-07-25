# AGENTS.md

These instructions are for coding agents working in this repository (Codex, Claude Code, Qodo, Cursor, etc.).

## Feedback Loop

If I ever correct you, add the correction to `FEEDBACK.md` so it never happens again.

## Mission

Build clean, modern macOS and Linux desktop apps in Rust with Tauri, while keeping each platform's behavior and visuals native-feeling.

## Non-negotiables

- Prefer simple, legible UI and obvious interaction flows.

## UI direction

- Very CLI like a terminal, but in a hacky kind of cool way.
- Keyboard shortcuts for every primary action.
- ASCII art is welcome when it remains accessible and responsive.
- Aim for beautiful, easy to use, and hacker-oriented.
- Center the UI and make it adapt cleanly at every supported window width.

## Architecture

- Backend: Rust via Tauri v2 (`src-tauri/`)
- Frontend: static HTML/CSS/JS (`ui/`) — no Node.js or bundler required
- IPC: Tauri commands (`#[tauri::command]`) invoked from JS via `window.__TAURI__.core.invoke()`
- Configuration: `src-tauri/tauri.conf.json`

## Iconography

- Prefer SF Symbols for macOS-specific iconography and portable SVGs with compatible licensing for shared macOS/Linux UI.
- Store symbol exports in `assets/symbols/` (typically SVG), and always provide a Linux-safe shared asset or text fallback.
- Keep icon weights and sizes consistent within a screen.

## Build and packaging expectations

- Use `scripts/build.sh` for host-native packaging.
- `.app` bundles are created by `scripts/build_macos_app.sh`.
- macOS bundles target Apple Silicon only (`aarch64-apple-darwin`); do not add Intel or universal build paths.
- Linux packages are created by `scripts/build_linux_app.sh` as `.deb` and AppImage artifacts on native x86-64 or ARM64 Linux.
- Build release AppImages on Ubuntu 22.04 or another deliberately chosen oldest-supported glibc baseline.
- Icon generation pipeline:
  - `assets/icons/AppIcon-1024.png`
  - `cargo tauri icon` generates the platform icon set in `src-tauri/icons/`
  - the bundle embeds `Contents/Resources/icon.icns`
- If icon source is missing, fallback chain is:
  - `scripts/generate_default_icon.swift`
  - existing `src-tauri/icons/icon.png`

## Commands agents should run

```bash
./scripts/dev.sh
./scripts/check.sh
./scripts/build.sh
./scripts/doctor.sh
```

## Repository skills

Use the focused skills in `.agents/skills/` when their descriptions match the task:

- `customize-tauri-app`
- `implement-tauri-feature`
- `design-terminal-ui`
- `secure-tauri-capabilities`
- `create-macos-app-icon`
- `diagnose-macos-toolchain`
- `verify-tauri-change`
- `package-macos-app`
- `tauri-local-data`
- `macos-native-integration`

## Change checklist

- Keep README/docs in sync with behavior.
- Keep scripts executable and cross-shell safe (`bash`, `set -euo pipefail`).
- Validate the affected host packaging path after refactors; preserve the verified macOS path when changing Linux support and vice versa.
- Frontend changes go in `ui/`, backend changes go in `src-tauri/src/`.
