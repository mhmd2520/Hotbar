# Hotbar

One keystroke to any project, with Claude Code already running.

**Double-tap Left Ctrl** anywhere in Windows to open the menu (Settings → Trigger switches it to Right Ctrl).
A number key opens that project in a maximized Windows Terminal running `claude`. Pressing Ctrl twice and then
a number within ~200 ms (the *blind chord*) opens the project without showing the menu at all. Ctrl ×2 again
closes the menu.

Clicking **Start menu → Hotbar** (or the desktop icon) opens the menu, starting Hotbar first if needed.
Autostart at logon passes `--background`, so it stays silent in the tray.

| Key in the menu | Does |
|---|---|
| `1`–`9` / click | open that project (`claude`, or the project's own Enter action) |
| `0` | Other… (plain PowerShell in `D:\`, or a folder picker, set in Settings) |
| `↵` | open the selected project |
| `⇧↵` / `⇧1`–`9` | `claude --continue` |
| `Alt↵` | `claude --resume` (session picker) |
| `Ctrl↵` | PowerShell only |
| `Ctrl⇧↵` | new window even if that project is already open |
| type letters | filter |
| `Ctrl ,` | settings |
| `Esc` / click outside | hide |

If a project's session window is already open, Hotbar focuses it instead of opening a duplicate.

## Config

`%APPDATA%\Hotbar\config.json` holds the projects, order, colours, theme, trigger timing and window mode.
Edit it in the Settings view (tray icon → Settings…) or by hand: Hotbar re-reads it the next time the menu opens,
and keeps the last good version if the JSON is broken. `%APPDATA%\Hotbar\hotbar.log` records start-up,
the keyboard hook and each open/close, which is the first place to look if the double-tap ever stops working. `HOTBAR_CONFIG` overrides the path (e.g. a OneDrive folder for sync).

## Build

Requires Rust (MSVC toolchain), Visual Studio C++ Build Tools and Node.

```powershell
npm install
npm run build          # installer: src-tauri\target\release\bundle\nsis\
cargo test --manifest-path src-tauri\Cargo.toml --lib
```

`ui\skins.js` and `ui\overlay.css` are generated from `..\design-explorer.html` (layouts, themes, button skins, effects)
by `python scripts\extract_ui.py`, so the explorer and the app stay identical: edit the explorer, then re-run it.
The fonts are bundled in `ui\fonts\` (`python scripts\bundle_fonts.py` re-downloads them), so the overlay needs no network.

## Start with Windows (elevated)

Low-level keyboard hooks don't see keys typed into admin windows unless Hotbar itself runs elevated.
From an admin PowerShell:

```powershell
.\scripts\install-autostart.ps1      # logon task with highest privileges, starts with --background
.\scripts\uninstall-autostart.ps1
```

When elevated, Hotbar starts terminals through the desktop shell, so Claude sessions never inherit admin rights.
