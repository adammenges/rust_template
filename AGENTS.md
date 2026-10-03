# Agent guide

Build approachable native Rust desktop apps with GPUI. Read
[docs/features.md](docs/features.md) and the implementation before changing behavior.
Record user corrections in `FEEDBACK.md` without deleting historical feedback.

## Architecture

- `app.json` is the identity source. `scripts/rename.py` synchronizes Cargo names;
  `build.rs` emits Rust identity constants. Never introduce another hardcoded identity.
- `domain.rs`: pure settings validation, structured state, and calculation policy.
- `app_paths.rs`: per-user paths and exclusive process lease.
- `persistence.rs`: validated Serde documents, bounded reads, atomic replacement.
- `background.rs`: bounded admission, CPU cancellation, serialized persistence,
  coalesced progress, terminal outcomes, and worker shutdown.
- `session.rs`: application-owned GPUI entity with committed state and worker owner.
- `app_window.rs`: window-owned navigation, draft fields, scratchpad, explicit handlers.
- `desktop_ui.rs`: shared visual tokens and pane, card, navigation, button controls.
- `text_input.rs`: native GPUI editor; preserve HEX attribution and IME semantics.
- `platform/`: native adapters. AppKit is main-thread-only and stays in `macos.rs`.
- `diagnostics.rs`: asynchronous structured logs with bounded retention.
- `main.rs`: CLI, startup wiring, window reuse, native events, shutdown handoff.

## UI conventions

Render directly in Rust using GPUI elements and `Render`. No HTML, JS, Node,
Tauri, or speculative UI framework. Use shared pane scaffolding, content width,
spacing, colors, and typography. Follow HEX’s gray sidebar (220 px, 144 px below
700 px), fixed 70 px pane header, 940 px shared content width, framed navigation
icons, restrained panels, and compact controls. Keep the full window canvas native, content
centered, and controls responsive at the 480 px minimum width. Primary actions
need keyboard shortcuts and visible hover states. Use gpui-symbols on macOS and
portable text/SVG fallbacks elsewhere. Keep a native titlebar, scroll below it,
and match its background to the canvas. Preserve editor selection, clipboard,
Unicode, IME, and undo behavior when changing input controls.

## Lifecycle and safety

One application entity owns workers and committed state; windows own drafts.
Closing the macOS window leaves the app and work running; menu bar/Dock reopen
it. Closing the Linux or preview window quits. Quit cancels CPU work and drains
accepted writes off the UI thread. The application stays alive until its shutdown worker finishes. The AppKit
quit gate uses NSTerminateLater for native requests and replies only after drain;
never depend on GPUI's 100 ms quit-future timeout for durable writes.
Native objects are released on the main thread. No blocking disk I/O during UI
handlers/rendering. Admission is bounded and nonblocking; progress is coalesced.
Do not hide persistence failures or replace malformed data with defaults on disk.
Settings errors block saves until external repair and restart; state errors block
runtime-state writes. Keep secrets and machine paths out of committed files.

Preview mode must remain deterministic and isolated: no data path discovery,
settings/log writes, worker creation, native tray installation, or real actions.
Do not claim Linux runtime/packaging verification from a macOS build. Linux uses
GPUI X11/Wayland with Vulkan; it does not use a GTK overlay or persistent daemon.

## Commands

```sh
./scripts/setup.sh
./scripts/doctor.sh
./scripts/dev.sh
./scripts/dev.sh --preview home
./scripts/check.sh
python3 scripts/test_preview.py  # after cargo build, graphical session required
./scripts/build.sh
```

Run formatting, meaningful tests, strict Clippy, a build, and native preview
inspection when available. Update README and the compact feature map with actual
verification limits. Package macOS only for `aarch64-apple-darwin`. Build Linux
release artifacts on Ubuntu 22.04 or a deliberately chosen oldest glibc baseline.
Keep scripts Bash with `set -euo pipefail`, executable, and safe with spaces.
Signing/notarization are optional parameters; never embed accounts or publish
artifacts without explicit authorization.

GitHub Actions workflows are intentionally omitted at the user’s request. Keep
validation local with `scripts/check.sh` and the host packaging/preview commands.
Do not reintroduce hosted workflows unless explicitly requested.
