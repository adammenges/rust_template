# Rust macOS `iced` Template

A GitHub template repository for building modern macOS desktop apps in Rust with [`iced`](https://github.com/iced-rs/iced).

## What you get

- `iced` starter UI with a clean, Apple-style baseline
- macOS `.app` bundling script
- icon pipeline (`AppIcon-1024.png` -> `AppIcon.icns`)
- default icon generator using SF Symbols (if no icon is provided)
- setup/dev/check scripts + `Makefile`
- CI workflow on macOS
- `AGENTS.md` instructions for Codex/Claude/Qodo-style coding agents

## Use as a GitHub template

1. Push this repository to GitHub.
2. In GitHub, open **Settings -> General -> Template repository** and enable it.
3. Click **Use this template** to create a new app repo.

GitHub reference: [Creating a repository from a template](https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-repository-from-a-template)

## Quick start

```bash
./scripts/setup.sh
./scripts/dev.sh
```

### Quality checks

```bash
./scripts/check.sh
```

### Build a macOS app bundle

```bash
./scripts/build_macos_app.sh
open dist/macos_iced_template.app
```

Optional overrides:

```bash
APP_NAME="My App" \
APP_EXECUTABLE="macos_iced_template" \
APP_BUNDLE_ID="com.example.myapp" \
UNIVERSAL=1 \
./scripts/build_macos_app.sh
```

## App icon workflow

1. Put a **1024x1024 PNG** at `assets/icons/AppIcon-1024.png`.
2. Build your app bundle with `./scripts/build_macos_app.sh`.
3. The script automatically generates `assets/icons/AppIcon.icns` and embeds it into the `.app`.

If no icon is found, the build script tries this fallback chain:

1. `scripts/generate_default_icon.swift` (SF Symbols-based icon)
2. macOS generic app icon extraction from `GenericApplicationIcon.icns`

## Project structure

- `src/main.rs`: main `iced` app shell
- `scripts/`: setup, checks, icon conversion, `.app` bundling
- `assets/icons/`: app icon source and generated `.icns`
- `assets/symbols/`: SF Symbol exports for in-app icon assets
- `.github/workflows/ci.yml`: macOS CI
- `AGENTS.md`: agent coding and design guidance

## Make targets

```bash
make setup
make dev
make check
make build-app
make clean
```
