# Application icon

Replace AppIcon-1024.png with a 1024 px PNG for your app. The macOS packaging script
uses sips/iconutil to produce Contents/Resources/icon.icns. If missing, the local
Swift renderer generates a neutral fallback. Linux packages use the PNG directly.
No Tauri icon generator or mobile icon sets are needed.
