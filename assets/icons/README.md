# App icon source

`AppIcon-1024.png` is the single source of truth for the application icon. Use a square 1024 × 1024 PNG with transparency where appropriate.

Regenerate Tauri's macOS, Linux, Windows, iOS, and Android icon set with:

```bash
make icons
```

The macOS and Linux build scripts regenerate icons when the source is newer than their generated bundle icon. If the source is missing, the scripts reuse `src-tauri/icons/icon.png`; macOS can also recreate the included default with `generate_default_icon.swift`.

Commit the generated files in `src-tauri/icons/` so a fresh checkout has a complete icon set.
