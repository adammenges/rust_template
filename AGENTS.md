# AGENTS.md

These instructions are for coding agents working in this repository (Codex, Claude Code, Qodo, Cursor, etc.).

## Mission

Build clean, modern macOS desktop apps in Rust with `iced`, while keeping behavior and visuals aligned with the Apple ecosystem.

## Feedback Loop
If I ever correct you, add the correction to `FEEDBACK.md` so it never happens again.

## Non-negotiables

- Keep this template macOS-first.
- Prefer simple, legible UI and obvious interaction flows.
- Make changes production-ready: formatting, linting, and tests/checks should pass.

## UI direction

- Very CLI like a terminal, but in a hacky kind of cool way.
- Keyboard shortcuts for everything.
- ASCII art is nice
- Beautiful, easy to use, hacker
- Center the UI, it should always look good regardless of window width

## Iconography

- Prefer SF Symbols for in-app iconography.
- Store symbol exports in `assets/symbols/` (typically SVG).
- Keep icon weights and sizes consistent within a screen.
- App icon source of truth is `assets/icons/AppIcon-1024.png`.

## Build and packaging expectations

- `.app` bundles are created with `scripts/build_macos_app.sh`.
- Icon generation pipeline:
  - `assets/icons/AppIcon-1024.png`
  - `scripts/make_icns.sh` -> `assets/icons/AppIcon.icns`
  - bundle embeds `Contents/Resources/AppIcon.icns`
- If icon source is missing, fallback chain is:
  - `scripts/generate_default_icon.swift`
  - macOS `GenericApplicationIcon.icns` extraction

## Commands agents should run

```bash
./scripts/dev.sh
./scripts/check.sh
./scripts/build_macos_app.sh
```

## Change checklist

- Keep README/docs in sync with behavior.
- Keep scripts executable and cross-shell safe (`bash`, `set -euo pipefail`).
- Validate macOS packaging still works after refactors.
- Do not add unrelated frameworks for UI when `iced` can solve it.
