# Native Starter

A reusable native Rust desktop template modeled on HEX's GPUI architecture.
The interface is Rust all the way down: GPUI elements, application entities,
`Render`, and explicit event handlers. No webview, IPC bridge, Node, or bundler.

The example has Home and Settings panes, a Unicode scratchpad, persisted workspace
name and spacing density, and a real cancellable CPU calculation with progress,
results, and an explicit failure demonstration. macOS has a native menu-bar entry.

## Develop

Requirements: Rust 1.95.0 with rustfmt/Clippy and Python 3. On Apple Silicon macOS,
install Command Line Tools (`xcode-select --install`). GPUI uses Apple's native
renderer; no audio, voice engine, credentials, or cloud setup is required.

```sh
./scripts/setup.sh
./scripts/doctor.sh
./scripts/dev.sh
./scripts/check.sh
```

The lockfile is committed. `setup.sh` uses rustup when available, checks the host,
and fetches locked dependencies. A Homebrew Rust installation also works if it
matches the pinned version and includes Clippy/rustfmt.

On Ubuntu 22.04, first run `./scripts/install_linux_dependencies.sh`. Linux builds
use GPUI's X11 and Wayland features and require a Vulkan-capable GPU/driver stack.
Linux implementation and packaging scripts are present but have
**not been compiled or exercised on Linux in this rewrite**. Other operating
systems have no adapter. macOS packaging is Apple Silicon only.

```sh
./scripts/dev.sh --preview home
./scripts/dev.sh --preview settings
./scripts/dev.sh --preview home --preview-width 480 --preview-height 600
./scripts/dev.sh --preview error --quit-after-ms 3000
./scripts/dev.sh --data-dir /tmp/my-isolated-starter
```

Preview uses fixed settings/progress/results and in-memory text fields. It never
discovers or creates user storage, installs a status item, or starts workers.
Save, Run, Cancel, and density changes are disabled. Text editing/navigation work
in memory. The error preview renders the real error presentation with a fixture.
Use `--data-dir` for isolated **real** persistence and worker checks.

Keyboard: Command on macOS, Ctrl on Linux. `1` Home, `2` Settings, `R` Run,
`.` Cancel, `Shift-R` Try failure, `S` Save, `Shift-D` Toggle draft density, `L` Focus scratchpad, `W` Close,
`Q` Quit. Text fields use native editing/clipboard bindings and input methods.

## Create a new app

Copy this repository into a new directory, excluding `.git`, `target`, `dist`, and
local `.env` files; initialize a new Git repository if desired. Then run:

```sh
python3 scripts/rename.py \
  --package my-workbench \
  --executable my-workbench \
  --display-name 'My Workbench' \
  --identifier org.example.my-workbench \
  --storage-name my-workbench
cargo check
./scripts/check.sh
./scripts/dev.sh
./scripts/build.sh
```

`app.json` centralizes package, executable, display name, reverse-DNS application
identifier, and storage directory name. The rename tool validates them and updates
Cargo's package/bin/default-run entries. `cargo check` refreshes the root package
entry in Cargo.lock. Rust identity constants and package metadata derive from that
file. `--check` detects drift. Package and executable names may differ. Storage
names are path components, never arbitrary paths.

Update the version in Cargo.toml, README/product copy, and your copyright notice;
replace `assets/icons/AppIcon-1024.png` with a 1024 px PNG. Keep required third-party
notices. Changing the storage name selects a new data directory and does not
migrate an existing installation. Decide migration policy before distributing a
renamed app. Remove or replace the demonstration calculation/scratchpad as your
app grows; keep domain policy, platform details, and storage ownership separated.

## Architecture and lifecycle

```text
CLI / startup -> application-owned Session -> window-local drafts and navigation
                          |                       |
                          |                 shared GPUI controls + TextInput
                          +-> cancellable calculation worker
                          +-> serialized atomic persistence worker
                          +-> typed committed settings/runtime result
AppKit status item -> coalesced Open/Quit intents -> application event loop
```

`domain.rs` contains pure policy and Serde models. `persistence.rs` owns document
serialization and atomic storage. `background.rs` owns threads and bounded
channels; `session.rs` projects terminal outcomes into observable app state.
`desktop_ui.rs` owns consistent pane headers, typography, width, spacing, and
controls. The window owns draft settings and its scratchpad. `platform/macos.rs`
owns the native status item and retains its target/menu until explicit teardown.

On macOS, closing the window keeps the application and accepted work alive.
Open from the menu bar or Dock to recreate/focus it. Quit cancels calculation,
drains accepted persistence writes, removes the native status item, and joins both
workers. Linux has no tray or service; closing its window quits. Preview closes
quit on both platforms. No worker is owned by a window.

There is one calculation at a time. Job admission capacity is one; the storage
queue holds at most two requests; terminal replies hold four. UI handlers use
`try_send`, never wait. Progress is an atomic latest value, not a growing queue.
The worker itself rejects duplicate starts until its terminal result is consumed;
rejected requests cannot reset another job’s cancellation or progress. Unexpected
worker exit is surfaced and further admission stops until restart.
Cancellation is checked every chunk (100,000 samples) with a 20 ms demo pacing
interval. Terminal calculation failure is explicit and retryable. Accepted saves
drain on quit even if the UI reply receiver is gone. The application waits asynchronously for its shutdown thread **before** asking
GPUI to quit. On macOS an AppKit quit gate also defers Dock/system termination
with `NSTerminateLater` and replies after work and logs finish. GPUI's quit futures
have a 100 ms timeout, and AppKit termination need not return from `run()`, so
neither is used as the durability boundary. Linux also joins the finished thread
after the event loop returns. OS-level disk stalls can still delay process exit; force-kill can lose
uncommitted work. Calculation results are persisted only after completion.

Storage:

- macOS: `~/Library/Application Support/<storage_name>/`
- Linux: `$XDG_DATA_HOME/<storage_name>/`, normally `~/.local/share/<storage_name>/`
- `settings.json`, `state.json`, `instance.lock`, and `logs/process.*.jsonl`

The directory is owner-only on Unix. An exclusive lease prevents competing
writers. Missing files use defaults; malformed, oversized, or future-version
settings are preserved and reported. A settings load failure blocks saves until
you repair/move the file externally and restart. A state load failure blocks state
writes, while calculations still work in memory. Existing shared directories passed
as `--data-dir` are rejected without changing their permissions; use a dedicated
private directory. Symlinked roots/locks and nonregular settings/state documents
(including FIFOs) are rejected; existing files are preserved. Validated writes use a sibling
file, file sync, atomic rename, and parent-directory sync. A failed save keeps the
committed UI settings active and exposes its error. If directory sync fails after
rename, the disk commit is uncertain: restart to reload. A crashed `.pending`
file is ignored and replaced on the next explicit save. Settings show unsaved,
saving, and saved feedback. Load errors and the latest action failure remain
visible together; saving preferences does not replace an active job’s status.

Diagnostics are structured Tracing JSON with `RUST_LOG` filtering, an asynchronous
1,024-line lossy queue, daily rotation, and seven retained files. Queue pressure
may drop diagnostic lines, never application outcomes. Preview logs to stderr.
No credentials are needed; `.env`, signing keys, and build outputs are ignored.

## Package

```sh
./scripts/build.sh
open 'dist/Native Starter.app'                 # macOS
./scripts/build.sh --deb-only                 # native Linux: .deb only
APPIMAGETOOL=/path/to/appimagetool ./scripts/build.sh  # native Linux: .deb + AppImage
```

macOS builds a release `aarch64-apple-darwin` executable, creates Info.plist from
identity metadata, converts the icon with `sips`/`iconutil`, includes attribution,
ad-hoc signs, and verifies identity/resources/ARM64/signature. Default artifacts
are for local use. Optional distribution inputs are:

```sh
SIGNING_IDENTITY='Developer ID Application: Your Name (YOURTEAM)' \
NOTARY_PROFILE='your-keychain-profile' ./scripts/build.sh
```

No signing account/team is embedded. Notarization is opt-in via a Keychain profile;
there is no publishing, updater feed, bucket, or release workflow. Validate your
signing/notarization on your own account before distribution.

Linux packaging targets native x86-64 or ARM64, generates a `.desktop` entry/icon,
and builds a `.deb` with runtime dependencies. For AppImages, supply a trusted
host-architecture `appimagetool` on PATH or via `APPIMAGETOOL`; the script does not
download or execute a remote packaging tool. It bundles linked libraries except
glibc and host graphics drivers. Host Vulkan drivers remain required. Release
AppImages must be built on Ubuntu 22.04 (glibc 2.35), or a deliberately chosen older
baseline, and tested on the destination GPU/display stack. The Debian package
also declares glibc >= 2.35. Build and validate packages locally on the appropriate
host. GitHub Actions workflows are intentionally omitted; use `./scripts/check.sh`
for formatting, tests, and strict Clippy. AppImage generation is an explicit local
packaging step.

## Repeatable native smoke checks

After `cargo build --locked`, run this in a graphical macOS or Linux session:

```sh
python3 scripts/test_preview.py
# Or exercise the packaged macOS executable:
python3 scripts/test_preview.py --executable 'dist/Native Starter.app/Contents/MacOS/native-starter'
```

It checks all three fixtures at different viewport sizes, rejects conflicting
CLI flags, observes startup/window/quit/shutdown ordering, and verifies that real
user storage remains unchanged. A timeout fails the check. It does not replace
visual inspection, physical IME testing, or live persistence/workflow testing.
Use `--executable` with your new executable path after renaming the template.
See `docs/verification.md` for the exact checks and native inspection limits.

## What differs from HEX

The GPUI version/configuration follows HEX: 0.2.2 with defaults off, `font-kit`,
plus X11/Wayland on Linux; macOS uses objc2/AppKit and gpui-symbols 0.6.1. The full
MIT-licensed text input is adapted with attribution and Linux Ctrl bindings.
The shell follows HEX’s gray visual tokens, 220 px sidebar, framed navigation
icons, fixed 70 px pane headers, 940 px shared content width, compact controls,
and pane/panel rendering. Below 700 px the sidebar narrows to 144 px so the
480 px minimum remains usable. Home and Settings keep generic example content.

GPUI's **built-in** `runtime_shaders` feature is enabled by default so Command Line
Tools suffice; HEX's default precompiles the same framework shaders using Xcode's
Metal tools. This adds no custom shaders. With full Xcode/Metal installed, use
`GPUI_PRECOMPILED_SHADERS=1 ./scripts/build.sh` or `cargo build --no-default-features`.
Runtime shaders add framework shader compilation at startup.

The starter has no recognizer, audio queues, hotkey event tap, GTK overlay, engine
service, global settings atomics, API daemon, NDJSON event database, update engine,
or release accounts. Two workers and one application entity are enough here;
Tracing plus a typed last-result file replace HEX's voice observation pipeline.
Linux and macOS share the same application state because this template has no
independent voice service whose lifetime must outlive the Linux UI.

See [AGENTS.md](AGENTS.md), [feature map](docs/features.md),
[verification](docs/verification.md), and [third-party notices](THIRD_PARTY.md).
