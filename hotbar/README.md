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
| `Ctrl⇧↵` | new session even if that project is already open |
| type letters | filter |
| `Ctrl ,` | settings |
| `Esc` / click outside | hide |

If a project's session window is already open, Hotbar focuses it instead of opening a duplicate.
Settings → Launch → "Sessions open in" decides where a new session goes:

- **One window: all projects as tabs**: every project is a tab in one shared window, mixed side by side;
  reopening a project switches to its tab, even when another tab is in front
- **A window per project**: a forced second session opens as a tab beside the project's first
- **A new window every time**

## Config

`%APPDATA%\Hotbar\config.json` holds the projects, order, colours, theme, trigger timing and window mode.
Edit it in the Settings view (tray icon → Settings…) or by hand: Hotbar re-reads it the next time the menu opens,
and keeps the last good version if the JSON is broken. `%APPDATA%\Hotbar\hotbar.log` records start-up,
the keyboard hook and each open/close, which is the first place to look if the double-tap ever stops working. `HOTBAR_CONFIG` overrides the path (e.g. a OneDrive folder for sync).

## Build

Requires Rust (MSVC toolchain), Visual Studio C++ Build Tools and Node.

```powershell
npm install
$env:TAURI_SIGNING_PRIVATE_KEY = "$env:USERPROFILE\.tauri\hotbar.key"   # see Releases
npm run build          # installer + .sig: src-tauri\target\release\bundle\nsis\
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

Updating by hand: just run the new installer. It can't close an elevated Hotbar itself, so it ends the autostart task
before replacing files and starts it again afterwards (`src-tauri\windows\hooks.nsh`).

## Updates

Hotbar checks for a new version 30 s after it starts and then once a day (Settings → Updates → "Check
automatically" turns that off), and never installs on its own. The tag under the logo shows the running version,
or **vX available** when there's an update; clicking it opens **Settings → Updates**: installed version, last check,
the new version's notes, **Check for updates** and **Install vX** with a download progress bar. The tray menu
offers the same (**Install update vX**, or **Check for updates…**, which asks before installing).

Installing downloads the installer, verifies its signature against the public key in `tauri.conf.json` and runs
it silently in the background (`installMode: quiet`, i.e. NSIS `/S`): no wizard, no window, Hotbar just restarts.
It then opens its menu, with a toast "Hotbar updated to vY" (Settings → Updates also shows "Updated from vX to
vY"); untick "Open the Hotbar menu after an update" to have it come back silently in the tray instead. (Hotbar leaves `%APPDATA%\Hotbar\updating.json` for its next start to find; if
that start is still the old version, Settings → Updates says the update didn't finish.) Each check is logged in `hotbar.log`; a failed check (offline, say) is only
logged and shown in Settings, and retried within the hour.

After the update the autostart task, if there is one, brings Hotbar back elevated; without it, the installer
restarts Hotbar in the tray. `HOTBAR_UPDATE_ENDPOINT` overrides where `latest.json` is fetched from.

Testing an update end to end without publishing: build two versions with plain http allowed (release builds
otherwise refuse anything but https), serve the newer one locally and point Hotbar at it.

```powershell
'{"plugins":{"updater":{"dangerousInsecureTransportProtocol":true}}}' | Set-Content $env:TEMP\old.json
'{"version":"0.9.1","plugins":{"updater":{"dangerousInsecureTransportProtocol":true}}}' | Set-Content $env:TEMP\new.json
npx tauri build --config $env:TEMP\old.json   # install this one
npx tauri build --config $env:TEMP\new.json   # serve this one
python scripts\latest_json.py <dir>\Hotbar_0.9.1_x64-setup.exe --url-base http://127.0.0.1:8790/
npx http-server <dir> -p 8790 -a 127.0.0.1
[Environment]::SetEnvironmentVariable('HOTBAR_UPDATE_ENDPOINT', 'http://127.0.0.1:8790/latest.json', 'User')
```

Restart Hotbar (`schtasks /End /TN Hotbar; schtasks /Run /TN Hotbar`) so it reads the variable, then remove the
variable and reinstall a normal build afterwards.

## Releases

Hotbar fetches `releases/latest/download/latest.json` from this repo, i.e. from the newest release that isn't a
draft or pre-release, so every release from now on needs its `latest.json`. Updates are signed with the key in
`%USERPROFILE%\.tauri\hotbar.key`, which stays out of the repo: back it up, because installed copies only
accept updates signed with it (its public half is `plugins.updater.pubkey` in `tauri.conf.json`).

1. Bump the version in `src-tauri\tauri.conf.json`, `src-tauri\Cargo.toml` and `package.json`.
2. Build with the key set (above). Without `TAURI_SIGNING_PRIVATE_KEY` the build stops at the signing step, after
   writing the installer. The key has no password; if you regenerate it with one, also set
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (and replace `pubkey`, which only works before the first release).
3. `python scripts\latest_json.py src-tauri\target\release\bundle\nsis\Hotbar_X.Y.Z_x64-setup.exe --notes "…"`
   writes `latest.json` next to the installer.
4. Publish all three as one release:

```powershell
git tag -a vX.Y.Z -m "Hotbar vX.Y.Z"; git push origin vX.Y.Z
$n = "src-tauri\target\release\bundle\nsis"
gh release create vX.Y.Z --verify-tag -t "Hotbar vX.Y.Z" `
  "$n\Hotbar_X.Y.Z_x64-setup.exe" "$n\Hotbar_X.Y.Z_x64-setup.exe.sig" "$n\latest.json"
```
