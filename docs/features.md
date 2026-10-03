# Implemented feature map

This map describes shipped example behavior, not a roadmap or proof of native tests.
Both roots share `app_window.rs`, `session.rs`, and `desktop_ui.rs`.

| Capability | Entry points / behavior | Platform | Evidence and limits |
|---|---|---|---|
| Shell | Launch; Home/Settings navigation; Cmd/Ctrl 1/2; HEX gray sidebar and controls; fixed 70 px header/shared 940 px content; responsive sidebar and scrollable body, 480 px minimum | macOS; Linux adapter | Native preview inspection in verification.md; compilation alone is not interaction proof |
| Text editing | Settings name; multiline Home scratchpad; pointer/word/line selection, clipboard, undo/redo, IME | GPUI on both | `text_input.rs` Unicode/selection/composition tests; physical IME candidates and Linux clipboard need native checks |
| Preferences | Draft name/density, unsaved/saving/saved feedback, Save or Cmd/Ctrl S; successful save changes Home and both panes' spacing; reload on startup | Both | Domain validation, atomic round-trip, invalid-file preservation tests; native save/restart evidence separate |
| Background work | Run/Cmd/Ctrl R -> chunked CPU checksum -> progress -> result; Cancel/Cmd/Ctrl .; Try failure/Cmd/Ctrl Shift-R | Both | Worker admission/cancellation/failure/completion and unexpected-exit tests; no file/network workload implied |
| Runtime state | Last successful result retained in state.json; failed/cancelled jobs do not replace it | Both | State round-trip test; corruption preserves file and blocks writes until repair/restart |
| Recovery | Visible load/save/job errors; malformed settings never overwritten; external file repair then restart | Both | Malformed/future/oversized/nonregular-document and write-failure/recovery tests; directory-sync failure after rename has uncertain disk commit |
| Native ownership | Close leaves app/work alive; menu bar Open/Quit; Dock reopens window | macOS | AppKit target/action implementation; native evidence and remaining gaps in verification.md |
| Window exit | Closing last window cancels work and quits; no background daemon or tray | Linux; preview on both | Linux not locally executed; source and CI configuration are not host proof |
| Quit | Cmd/Ctrl Q/native menu -> cancel CPU -> drain saves -> release native item -> application waits for worker drain before native termination | Both | Worker drain test; GPUI 100 ms timeout avoided with pre-termination drain and AppKit quit gate; OS disk stalls remain possible |
| Diagnostics | JSON logs, RUST_LOG, async bounded queue, daily rotation, seven files | Both | Startup/shutdown records; preview uses stderr and creates no log directory |
| Preview | --preview home/settings/error; fixed fixtures; text/nav only; optional --quit-after-ms | Both | `scripts/test_preview.py` checks three viewport fixtures, lifecycle, CLI rejection, and storage isolation; fixtures do not prove actual work/persistence/native integration |
| Identity | app.json -> rename tool -> Cargo, Rust constants, plist/desktop/package metadata | Both | Isolated renamed-identity/native-metadata test; changing storage name does not migrate data |
| Packaging | build.sh -> ARM64 .app; native Linux .deb/AppImage | macOS; Linux adapter | See verification.md for executed builds; signed distribution/notarization/publishing not exercised |

Implementation entry points: `main.rs`, `app_window.rs`, `domain.rs`,
`background.rs`, `persistence.rs`, `platform/macos.rs`. Focused tests live beside
behavior; `scripts/test_template.py` covers rename/package metadata isolation and rejected-input
preservation. `scripts/test_preview.py` is an opt-in native smoke check.
