# Third-party source

`src/text_input.rs` is adapted from HEX's MIT-licensed implementation at
`src/text_input.rs` in https://github.com/anomalyco/hex, inspected at revision
`faa2192110e668d5e491443f85a997cb851e0cf2`. Copyright (c) 2026 Christopher Langton.
The upstream license is preserved in `licenses/HEX-MIT.txt`. The AppKit
target/action structure in `src/platform/macos.rs` and visual vocabulary in
`src/desktop_ui.rs` are also adapted from HEX.
Packagers include these notices in application artifacts.

The editor retains native GPUI input-method integration, UTF-16/UTF-8 conversion,
grapheme and word navigation, pointer selection, wrapped multiline input,
clipboard, and bounded undo/composition history. Linux adds conventional Ctrl
bindings. This module is intentionally larger than the shell: replacing it with
a decorative input would lose behavior needed by future apps.

GPUI and gpui-symbols are third-party framework dependencies, under their own
licenses. Cargo.lock records the full dependency graph. Review all dependency
licenses before distributing your derivative app.
