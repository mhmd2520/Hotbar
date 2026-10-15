# Hotbar

Double-tap **Left Ctrl** anywhere in Windows, press a number, and that project opens in a maximized
Windows Terminal with Claude Code already running.

| Folder | What's in it |
|---|---|
| [`hotbar/`](hotbar/) | The app: Tauri 2 (Rust + WebView2) tray app, installer, autostart scripts. See [hotbar/README.md](hotbar/README.md). |
| [`design-explorer.html`](design-explorer.html) | The design playground the app's look was picked in: 15 layouts, themes, button skins, gestures. `hotbar/scripts/extract_ui.py` copies its styles into the app. |
| [`branding/`](branding/) | Logo concepts and the app icon source (`hotbar-icon.png`). |

Installers are attached to the [GitHub releases](../../releases).
