# Interface symbol assets

Use this folder for local SVG symbols used inside the app UI. Shared macOS/Linux controls must use portable assets with compatible distribution terms.

For macOS-only UI:

1. Open Apple's SF Symbols app.
2. Export symbols as SVG.
3. Save them in this directory with stable names.
4. Give them a `.macos.svg` suffix and provide a portable Linux asset or text fallback.

Keep icon style consistent:

- Use a consistent optical weight within each screen.
- Provide an accessible text label or `aria-label` for interactive symbols.
- Prefer monochrome or a restrained palette so symbols inherit UI state cleanly.

Review Apple's SF Symbols license before distribution. Do not include an SF Symbol export in a Linux package unless its license explicitly permits that use.
