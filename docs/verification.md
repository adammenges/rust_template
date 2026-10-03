# Rewrite verification — 2026-10-03

Observed on Apple Silicon macOS 26.6.2, Homebrew Rust/Cargo 1.95.0, Python 3.14.7,
and Command Line Tools. No full Xcode Metal compiler is installed.
The repository started with a clean tracked working tree. Historical FEEDBACK,
license, and neutral icon source were retained. Obsolete Tauri source/UI/config,
mobile icon sets, Tauri-specific agent skills, and obsolete documentation were
replaced. HEX was read only; its unrelated untracked `prompt.md` was untouched.
Source study included its manifest, main/startup and macOS/Linux owners,
app_window, desktop_ui, text_input, app_settings, app_paths, status_item, events,
agent guide, and feature-map index. Implementation was used to establish behavior.

## Executed checks

- `./scripts/check.sh`: identity validation; isolated renamed-template and native
  metadata checks; Bash syntax; `cargo fmt --all -- --check`; 23 Rust tests;
  `cargo clippy --locked --all-targets -- -D warnings`.
- Tests cover domain validation/chunk equivalence; settings/state round trips,
  invalid/oversized-file preservation, atomic-write cleanup; process-lease
  exclusivity/release and refusal to change an existing shared directory's mode;
  real worker completion/cancellation/failure and drain of both accepted settings
  and state writes; 13 inherited editor composition/selection/Unicode checks.
- `cargo check`, `cargo build --locked`, `./scripts/doctor.sh`, `git diff --check`.
- `./scripts/build.sh`: optimized Apple Silicon-only `.app`; generated plist and
  icon; included MIT attribution; ARM64, plist, identity/resource, and ad-hoc
  signature verification. Artifact: `dist/Native Starter.app`.
- `swift scripts/generate_default_icon.swift target/qa-fallback-icon.png` and
  `sips` dimension inspection: fallback renders a 1024×1024 PNG. The inherited
  unsupported tint API and Retina-sized output were fixed.
- Reproducible development preview and packaged release previews launched.
  Automatic native quit preview exited with code 0 and emitted, in order:
  startup, native_quit_deferred, shutdown_requested, shutdown_complete.
  Cocoa delayed selector scheduling is required here: invoking NSTerminateLater
  while occupying libdispatch's serial main queue blocked GPUI's poller.
- CLI rejects preview + data-dir before creating that directory.
- Final `./scripts/dev.sh --preview home --quit-after-ms 800` exited 0 through
  the native quit gate. Hashes of isolated real settings/state were unchanged,
  and the normal per-user storage directory was absent before and after preview.

## Native UI observations

Screenshots were inspected through Computer Use for release Home/Settings at
920×800 and 480×600 points. Shared pane headers/content, wrapping controls,
scrolling, native SF Symbols, and the titlebar/navigation separation were visible.
Cmd-1/2 navigation works. Cmd-2 focuses the name field; Cmd-L focuses and scrolls
to the multiline scratchpad. Selecting all, pasting Unicode, and undo restored
original selected text. Computer Use's paste call reported a clipboard-read
notification timeout, but the screenshot confirmed the Unicode paste succeeded;
undo was then separately exercised and observed.

Real application checks used only the dedicated `target/qa-data` directory:

- Changed the workspace name and compact density, saved with Cmd-S, inspected
  settings.json, and observed both changes on Home.
- Ran actual computation; observed progress; verified its checksum in state.json.
- Cancelled another calculation; observed terminal cancellation and preservation
  of the prior result. Ran the failure example; observed the error banner and
  structured job_failed record.
- Another process using the same data directory was rejected before startup.
- Cmd-W closed the window without ending the process. Subsequent Computer Use
  activation reopened a new window. This checks close/reopen, not a menu-bar click.
- Quit, restarted, and observed the saved name, compact layout, and last result.
- Quit immediately after starting work; logs confirmed job_cancelled,
  workers_stopped, and shutdown_complete before the process exited.

Previews used no Store/workers/status item. All real-app storage checks were
isolated; normal per-user settings were not used. Test artifacts and package
outputs remain Git-ignored. No signing account, credentials, or publishing was used.

## Limits and known gaps

- Linux code, system dependency installation, X11/Wayland/Vulkan behavior,
  `.deb`/AppImage packaging, and Linux CI have **not run locally**. Their presence
  is an explicit adapter/build contract, not validated Linux support. AppImage
  library selection needs host testing; validate Debian runtime dependencies on
  the oldest chosen distribution before release.
- The macOS status item installed without an error, but its Open/Quit menu clicks
  were not exercised. Computer Use exposed the app window/application menu, not
  the status item's menu. Dock/system termination was exercised through AppKit's
  native terminate request in a preview; physical Dock-menu and OS logout/restart
  interaction remain unverified.
- Physical IME candidate windows, drag selection, native character palette,
  Linux clipboard, and every keyboard layout were not tested. Composition and
  Unicode unit checks are not physical IME proof.
- GPUI content was absent from the native accessibility tree observed here;
  screen-reader semantics and complete Tab-based control traversal are not
  established. Explicit action/focus shortcuts work. This remains an accessibility
  limitation of this starter and its current framework integration.
- GPUI logged a transient `window not found` for late native input after Cmd-W.
  The app remained alive and reopened; the framework-level message is retained
  in diagnostics rather than suppressed.
- Full-Xcode precompiled-shader mode, Developer ID signing, notarization,
  stapling, older macOS releases (the plist declares 12.0), and release publishing
  were not exercised. The Homebrew toolchain emitted linker warnings about some
  objects built for newer macOS versions; this host build does not prove the
  declared oldest deployment target. Use a suitable Rust/Xcode toolchain and test
  your chosen minimum OS before distribution.
- Disk-full, fsync failure, forced process death during commit, filesystem stalls,
  and installed Linux package upgrades were not fault-injected. Accepted writes
  can delay exit if the OS stalls; force-kill can lose uncommitted work.

## Robustness and HEX visual fidelity follow-up — 2026-10-03

The shell now uses HEX’s actual gray palette, 220 px sidebar, framed 12 px SF
Symbols, 38 px navigation rows, fixed 70 px headers, 20 px semibold pane titles,
940 px shared header/body width, 10 px panel corners, and 32/34 px controls/inputs.
The sidebar becomes 144 px below a 700 px viewport; this responsive adaptation
keeps the generic 480 px minimum. Settings uses a compact toggle and a header Save
action, with unsaved/saving/saved feedback. Independent load errors and the latest job/save error
are rendered together instead of hiding later failures behind the first one.

This pass fixed duplicate job admission resetting cancellation/progress;
unobservable unexpected worker exit; FIFO/symlink document handling; transient
public storage-directory creation; dangling pending-file recovery; and the size
limit excluding the trailing newline. Identity checks now reject missing/extra/
wrong-type fields and missing Cargo identity entries, preserve files on rejected
renames, and validate direct Cargo builds too. macOS packaging rejects unsupported
arguments and invalid ad-hoc/notarization combinations before compiling; Linux
packaging reports missing libraries rather than silently omitting them.

Executed checks in this follow-up:

- `./scripts/check.sh`: **37 Rust tests passed**, isolated Python identity/metadata
  checks passed, formatting/Bash syntax passed, strict Clippy passed.
- New tests include duplicate admission/cancellation/re-admission, full reply queue
  shutdown and pending-write drain, persistence failure/recovery without replacing
  committed settings, unexpected worker exit/admission refusal, stale crash files,
  private file modes, future/unknown settings, symlinked root/lock/documents,
  directories/FIFOs, exact 64 KiB serialization boundary, Unicode name limits and
  control characters, invalid CLI combinations, and minimum fixture dimensions.
- `cargo build --locked`, `./scripts/doctor.sh`, and `git diff --check` passed.
- `./scripts/build.sh` rebuilt the ARM64 release `.app`; signature, metadata,
  executable architecture, icon, and included attribution checks passed.
- `python3 scripts/test_preview.py`: Home 480×480, Settings 920×800, Error 1440×900
  each exited 0; startup, application_ready, window_opened, native deferred quit,
  shutdown_requested, and shutdown_complete were observed. Default user storage
  was absent before and after; conflicting CLI flags created no directory.
- The same native smoke checks passed against the final packaged executable.
  Ad-hoc signing plus NOTARY_PROFILE and unsupported macOS build arguments were
  both rejected before compilation. No preview processes remained after checks.
- An actual isolated template was renamed to package `example-workbench`, executable
  `workbench`, display name `Example Workbench`, with distinct identity/storage.
  `cargo check --offline` compiled it from a directory containing spaces.
- Computer Use screenshots showed the new HEX-style Home and Settings at 920×800;
  Cmd-2 selected Settings and focused its name input. Later capture attempts after
  app restarts failed with `cgWindowNotFound`, including retries with reset bindings,
  separate QA identities, and the release bundle. The final narrow-layout screenshot,
  final save-feedback interaction, and error layout therefore remain unverified
  visually; successful fixture launches are lifecycle proof only. Earlier narrow
  observations above describe the original rewrite, before the sidebar change.
- Linux container verification was attempted with `docker info`; the local Docker
  daemon/socket was unavailable. No Linux build/runtime/package claim is added.

The physical IME, accessibility, tray clicks, signing/notarization, older-OS,
disk-full/fsync/force-kill, and filesystem-stall limits listed above still apply.
