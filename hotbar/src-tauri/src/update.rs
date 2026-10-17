//! Self-update from the endpoint in tauri.conf.json (`plugins.updater`), or `HOTBAR_UPDATE_ENDPOINT`.
//! Checked 30 s after start and then once a day; an update found is offered in the tray menu and is
//! only installed when that item is clicked. The download is verified against the bundled public key.

use crate::log;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime};
use tauri::menu::MenuItem;
use tauri::{AppHandle, Manager, Url, Wry};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::{Update, UpdaterExt};

pub const MENU_ID: &str = "update";
pub const CHECK_LABEL: &str = "Check for updates…";
const FIRST_CHECK: Duration = Duration::from_secs(30);
const INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
/// How often the background thread wakes to see whether a check is due (wall clock, so a laptop that
/// slept through the day still checks soon after waking, and a failed check is retried within the hour).
const TICK: Duration = Duration::from_secs(60 * 60);
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);
/// The plugin applies its timeout to the whole installer download, not just to connecting.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(10 * 60);

pub struct Updates {
    item: MenuItem<Wry>,
    /// Version the tray item currently offers ("Install update vX"), if any.
    offered: Mutex<Option<String>>,
    /// Held while checking or installing (never across a dialog), so a click and the daily check
    /// never overlap.
    work: Mutex<()>,
}

/// Manage the state behind the tray item and start the background checks.
pub fn start(app: &AppHandle, item: MenuItem<Wry>) {
    app.manage(Updates { item, offered: Mutex::new(None), work: Mutex::new(()) });
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK);
        let mut last_ok = None;
        loop {
            let now = SystemTime::now();
            if due(last_ok, now) {
                if let Ok(_work) = app.state::<Updates>().work.try_lock() {
                    if check(&app, false).is_ok() {
                        last_ok = Some(now);
                    }
                }
            }
            std::thread::sleep(TICK);
        }
    });
}

/// Tray item clicked. Always checks again, so a release that was replaced since (or a retry after a
/// failed install) uses the current metadata. "Install update vX" installs vX straight away; anything
/// else (a plain check, or a newer version than the one offered) asks first.
pub fn on_menu(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let state = app.state::<Updates>();
        let offered = offered(&app).clone();
        let Ok(work) = state.work.try_lock() else { return };
        let found = check(&app, true);
        if let Ok(Some(u)) = &found {
            if offered.as_deref() == Some(u.version.as_str()) {
                return install_and_report(&app, work, u.clone());
            }
        }
        // dialogs are shown without holding `work`, so one left open doesn't block later clicks
        drop(work);
        match found {
            Ok(Some(u)) if confirm(&app, &u) => {
                if let Ok(work) = state.work.try_lock() {
                    install_and_report(&app, work, u);
                }
            }
            Ok(Some(_)) => {} // "Later": stays offered in the menu
            Ok(None) => message(&app, &format!("Hotbar is up to date (v{}).", version(&app))),
            Err(e) => message(&app, &format!("Couldn't check for updates:\n{e}")),
        }
    });
}

fn offered(app: &AppHandle) -> MutexGuard<'_, Option<String>> {
    app.state::<Updates>().inner().offered.lock().unwrap_or_else(|e| e.into_inner())
}

fn version(app: &AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Ask the endpoint; an update found is offered in the tray menu. A failed check leaves the menu as
/// it was, so an update already offered stays offered while offline. `to_install`: the result may be
/// installed, so it needs the installer arguments (which cost a `schtasks` call).
fn check(app: &AppHandle, to_install: bool) -> Result<Option<Update>, String> {
    set_item(app, "Checking for updates…", false);
    let result = updater(app, to_install).and_then(|u| tauri::async_runtime::block_on(u.check()).map_err(|e| e.to_string()));
    match &result {
        Ok(Some(u)) => {
            log(&format!("update: v{} available (running v{})", u.version, u.current_version));
            offer(app, &u.version);
        }
        Ok(None) => {
            log(&format!("update: up to date (v{})", version(app)));
            *offered(app) = None;
            set_item(app, CHECK_LABEL, true);
            set_tooltip(app, crate::TOOLTIP);
        }
        Err(e) => {
            log(&format!("update check failed: {e}"));
            let still = offered(app).clone();
            match still {
                Some(v) => offer(app, &v),
                None => set_item(app, CHECK_LABEL, true),
            }
        }
    }
    result
}

fn updater(app: &AppHandle, to_install: bool) -> Result<tauri_plugin_updater::Updater, String> {
    let mut b = app.updater_builder().timeout(CHECK_TIMEOUT);
    if let Some(url) = endpoint_override(std::env::var("HOTBAR_UPDATE_ENDPOINT").ok()) {
        b = b.endpoints(vec![url?]).map_err(|e| e.to_string())?;
    }
    if to_install {
        let task = autostart_task_exists();
        let handle = app.clone();
        b = b.installer_args(installer_args(task)).on_before_exit(move || {
            log(&format!("update: starting the installer and exiting (autostart task: {task})"));
            // the process exits without tearing down, which would leave a ghost icon in the tray
            set_tray_visible(&handle, false);
        });
    }
    b.restart_after_install(false).build().map_err(|e| e.to_string())
}

/// Download, verify the signature and hand over to the installer, which ends this process. Only
/// returns on failure, and reports it once `work` is released.
fn install_and_report(app: &AppHandle, work: MutexGuard<'_, ()>, mut update: Update) {
    update.timeout = Some(DOWNLOAD_TIMEOUT);
    set_item(app, &format!("Downloading v{}…", update.version), false);
    log(&format!("update: downloading v{} from {}", update.version, update.download_url));
    let result = tauri::async_runtime::block_on(update.download_and_install(|_, _| {}, || {}));
    drop(work);
    if let Err(e) = result {
        log(&format!("update install failed: {e}"));
        set_tray_visible(app, true);
        offer(app, &update.version);
        message(app, &format!("Couldn't install the update:\n{e}"));
    }
}

fn offer(app: &AppHandle, version: &str) {
    *offered(app) = Some(version.to_owned());
    set_item(app, &format!("Install update v{version}"), true);
    set_tooltip(app, &format!("Hotbar: update v{version} available"));
}

fn set_item(app: &AppHandle, text: &str, enabled: bool) {
    let item = &app.state::<Updates>().item;
    let _ = item.set_text(text);
    let _ = item.set_enabled(enabled);
}

fn set_tooltip(app: &AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id("hotbar") {
        let _ = tray.set_tooltip(Some(text));
    }
}

fn set_tray_visible(app: &AppHandle, visible: bool) {
    if let Some(tray) = app.tray_by_id("hotbar") {
        let _ = tray.set_visible(visible);
    }
}

fn confirm(app: &AppHandle, u: &Update) -> bool {
    app.dialog()
        .message(format!("Hotbar v{} is available (you have v{}).\nInstall it now?", u.version, u.current_version))
        .title("Hotbar")
        .buttons(MessageDialogButtons::OkCancelCustom("Install".into(), "Later".into()))
        .blocking_show()
}

fn message(app: &AppHandle, text: &str) {
    app.dialog().message(text).title("Hotbar").kind(MessageDialogKind::Info).blocking_show();
}

/// The elevated logon task from scripts\install-autostart.ps1. hooks.nsh restarts it after installing.
fn autostart_task_exists() -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new("schtasks")
        .args(["/Query", "/TN", "Hotbar"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .is_ok_and(|o| o.status.success())
}

/// What the installer does once it's done. With the autostart task, nothing: hooks.nsh restarts the
/// task, which brings Hotbar back elevated, and a second (unelevated) start from `/R` would race it
/// for the single instance. Without it, `/R` starts Hotbar again, silently in the tray.
fn installer_args(autostart_task: bool) -> &'static [&'static str] {
    if autostart_task {
        &[]
    } else {
        &["/R", "/ARGS", "--background"]
    }
}

/// `HOTBAR_UPDATE_ENDPOINT`, blank meaning unset. Release builds only accept https unless built with
/// `plugins.updater.dangerousInsecureTransportProtocol` (README → Updates → testing).
fn endpoint_override(value: Option<String>) -> Option<Result<Url, String>> {
    let v = value?;
    let v = v.trim();
    (!v.is_empty()).then(|| Url::parse(v).map_err(|e| format!("HOTBAR_UPDATE_ENDPOINT: {e}")))
}

fn due(last_ok: Option<SystemTime>, now: SystemTime) -> bool {
    match last_ok {
        None => true,
        // a clock set backwards counts as due rather than postponing the check indefinitely
        Some(t) => now.duration_since(t).map_or(true, |d| d >= INTERVAL),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_check_is_due_after_a_day_or_when_the_clock_went_back() {
        let t = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
        assert!(due(None, t));
        assert!(!due(Some(t), t + INTERVAL - Duration::from_secs(1)));
        assert!(due(Some(t), t + INTERVAL));
        assert!(due(Some(t), t - Duration::from_secs(60)));
    }

    #[test]
    fn endpoint_override_ignores_blank_and_reports_bad_urls() {
        assert!(endpoint_override(None).is_none());
        assert!(endpoint_override(Some("  ".into())).is_none());
        let url = endpoint_override(Some(" http://127.0.0.1:8790/latest.json ".into())).unwrap().unwrap();
        assert_eq!(url.as_str(), "http://127.0.0.1:8790/latest.json");
        assert!(endpoint_override(Some("not a url".into())).unwrap().is_err());
    }

    #[test]
    fn restart_is_left_to_the_autostart_task_when_there_is_one() {
        assert!(installer_args(true).is_empty());
        assert_eq!(installer_args(false), ["/R", "/ARGS", "--background"]);
    }
}
