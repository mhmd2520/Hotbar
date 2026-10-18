//! Self-update from the endpoint in tauri.conf.json (`plugins.updater`), or `HOTBAR_UPDATE_ENDPOINT`.
//! Checked 30 s after start and then once a day (unless `updates.autoCheck` is off); an update found is
//! offered in Settings → Updates, on the version tag and in the tray menu, and is only installed when
//! asked. The download is verified against the bundled public key.
//!
//! One `Status` describes it all: every change goes to the page as an `update` event and to the tray item.

use crate::log;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime};
use tauri::menu::MenuItem;
use tauri::{AppHandle, Emitter, Manager, Url, Wry};
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
/// Without a Content-Length, progress is reported every this many bytes.
const PROGRESS_STEP: u64 = 256 * 1024;
/// An install marker older than this is still reported, but doesn't open Settings any more.
const REOPEN_WITHIN: Duration = Duration::from_secs(10 * 60);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// The running version.
    pub current: String,
    /// The update on offer, from the last check that reached the endpoint (kept while a later one fails).
    pub available: Option<Release>,
    /// "idle" | "checking" | "downloading" | "installing" (the installer has started; Hotbar is exiting)
    pub phase: &'static str,
    /// Unix seconds of the last check that reached the endpoint.
    pub checked_at: Option<u64>,
    /// Why the last check or install failed; cleared by the next success.
    pub error: Option<String>,
    pub downloaded: u64,
    pub total: Option<u64>,
    /// This run was started by an update from this version.
    pub updated_from: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct Release {
    pub version: String,
    pub notes: Option<String>,
    /// RFC 3339
    pub date: Option<String>,
}

impl Status {
    fn initial(app: &AppHandle) -> Self {
        let current = app.package_info().version.to_string();
        Self { current, available: None, phase: "idle", checked_at: None, error: None, downloaded: 0, total: None, updated_from: None }
    }
}

impl Release {
    fn of(u: &Update) -> Self {
        let date = u.raw_json.get("pub_date").and_then(|d| d.as_str()).map(String::from);
        Self { version: u.version.clone(), notes: u.body.clone().filter(|n| !n.trim().is_empty()), date }
    }
}

pub struct Updates {
    item: MenuItem<Wry>,
    status: Mutex<Status>,
    /// Held while checking or installing (never across a dialog), so a click and the daily check
    /// never overlap.
    work: Mutex<()>,
}

/// Manage the state behind the tray item and start the background checks.
pub fn start(app: &AppHandle, item: MenuItem<Wry>) {
    app.manage(Updates { item, status: Mutex::new(Status::initial(app)), work: Mutex::new(()) });
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK);
        let mut last_ok = None;
        loop {
            let now = SystemTime::now();
            let auto = app.state::<crate::AppState>().store().cfg.updates.auto_check;
            if auto && due(last_ok, now) {
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

/// The page may ask before setup has managed the state (it preloads while the app starts).
pub fn status(app: &AppHandle) -> Status {
    match app.try_state::<Updates>() {
        Some(u) => u.status.lock().unwrap_or_else(|e| e.into_inner()).clone(),
        None => Status::initial(app),
    }
}

#[tauri::command]
pub fn update_status(app: AppHandle) -> Status {
    status(&app)
}

/// Settings → Updates → Check for updates. The result arrives as `update` events.
#[tauri::command]
pub fn check_update(app: AppHandle) {
    std::thread::spawn(move || {
        if let Ok(_work) = app.state::<Updates>().work.try_lock() {
            let _ = check(&app, false);
        }
    });
}

/// Settings → Updates → Install vX. Checks again first and installs only if the endpoint still offers
/// `version`; otherwise the new answer just shows up in the status for the user to look at.
#[tauri::command]
pub fn install_update(app: AppHandle, version: String) {
    std::thread::spawn(move || {
        let state = app.state::<Updates>();
        let Ok(work) = state.work.try_lock() else { return };
        if let Ok(Some(u)) = check(&app, true) {
            if u.version == version {
                let _ = install(&app, work, u);
            }
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
        let offered = status(&app).available.map(|r| r.version);
        let Ok(work) = state.work.try_lock() else { return };
        let found = check(&app, true);
        if let Ok(Some(u)) = &found {
            if offered.as_deref() == Some(u.version.as_str()) {
                if let Err(e) = install(&app, work, u.clone()) {
                    message(&app, &format!("Couldn't install the update:\n{e}"));
                }
                return;
            }
        }
        // dialogs are shown without holding `work`, so one left open doesn't block later clicks
        drop(work);
        match found {
            Ok(Some(u)) if confirm(&app, &u) => {
                let failed = state.work.try_lock().ok().and_then(|work| install(&app, work, u).err());
                if let Some(e) = failed {
                    message(&app, &format!("Couldn't install the update:\n{e}"));
                }
            }
            Ok(Some(_)) => {} // "Later": stays offered
            Ok(None) => message(&app, &format!("Hotbar is up to date (v{}).", app.package_info().version)),
            Err(e) => message(&app, &format!("Couldn't check for updates:\n{e}")),
        }
    });
}

fn lock_status(app: &AppHandle) -> MutexGuard<'_, Status> {
    app.state::<Updates>().inner().status.lock().unwrap_or_else(|e| e.into_inner())
}

/// Change the status, then show it on the tray item and send it to the page.
fn publish(app: &AppHandle, change: impl FnOnce(&mut Status)) {
    let s = {
        let mut s = lock_status(app);
        change(&mut s);
        s.clone()
    };
    let (text, enabled) = tray_item(&s);
    let item = &app.state::<Updates>().item;
    let _ = item.set_text(text);
    let _ = item.set_enabled(enabled);
    if let Some(tray) = app.tray_by_id("hotbar") {
        let tip = s.available.as_ref().map_or(crate::TOOLTIP.to_string(), |r| format!("Hotbar: update v{} available", r.version));
        let _ = tray.set_tooltip(Some(tip));
    }
    let _ = app.emit("update", &s);
}

fn tray_item(s: &Status) -> (String, bool) {
    match (s.phase, &s.available) {
        ("checking", _) => ("Checking for updates…".into(), false),
        ("downloading" | "installing", Some(r)) => match percent(s.downloaded, s.total) {
            Some(p) => (format!("Downloading v{}… {p}%", r.version), false),
            None => (format!("Downloading v{}…", r.version), false),
        },
        (_, Some(r)) => (format!("Install update v{}", r.version), true),
        _ => (CHECK_LABEL.into(), true),
    }
}

fn percent(done: u64, total: Option<u64>) -> Option<u64> {
    total.filter(|&t| t > 0).map(|t| (done.min(t) * 100) / t)
}

fn unix_now() -> Option<u64> {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).ok().map(|d| d.as_secs())
}

/// Ask the endpoint. A failed check leaves the offer as it was, so an update already offered stays
/// offered while offline. `to_install`: the result may be installed, so it needs the installer
/// arguments (which cost a `schtasks` call).
fn check(app: &AppHandle, to_install: bool) -> Result<Option<Update>, String> {
    publish(app, |s| s.phase = "checking");
    let result = updater(app, to_install).and_then(|u| tauri::async_runtime::block_on(u.check()).map_err(|e| e.to_string()));
    match &result {
        Ok(Some(u)) => log(&format!("update: v{} available (running v{})", u.version, u.current_version)),
        Ok(None) => log(&format!("update: up to date (v{})", app.package_info().version)),
        Err(e) => log(&format!("update check failed: {e}")),
    }
    publish(app, |s| {
        s.phase = "idle";
        match &result {
            Ok(found) => {
                s.available = found.as_ref().map(Release::of);
                s.checked_at = unix_now();
                s.error = None;
            }
            Err(e) => s.error = Some(e.clone()),
        }
    });
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
            publish(&handle, |s| s.phase = "installing");
            // the process exits without tearing down, which would leave a ghost icon in the tray
            set_tray_visible(&handle, false);
        });
    }
    b.restart_after_install(false).build().map_err(|e| e.to_string())
}

/// Download (reporting progress), verify the signature and hand over to the installer, which ends
/// this process. Only returns on failure, with `work` released.
fn install(app: &AppHandle, work: MutexGuard<'_, ()>, mut update: Update) -> Result<(), String> {
    update.timeout = Some(DOWNLOAD_TIMEOUT);
    log(&format!("update: downloading v{} from {}", update.version, update.download_url));
    publish(app, |s| {
        s.phase = "downloading";
        s.downloaded = 0;
        s.total = None;
        s.error = None;
    });
    let marker = Marker { from: update.current_version.clone(), to: update.version.clone() };
    let _ = serde_json::to_string(&marker).map(|json| std::fs::write(marker_path(), json));
    let (mut done, mut reported) = (0u64, None);
    let result = tauri::async_runtime::block_on(update.download_and_install(
        |chunk, total| {
            done += chunk as u64;
            let mark = percent(done, total).unwrap_or(done / PROGRESS_STEP);
            if reported != Some(mark) {
                reported = Some(mark);
                publish(app, |s| {
                    s.downloaded = done;
                    s.total = total;
                });
            }
        },
        || {},
    ))
    .map_err(|e| e.to_string());
    drop(work);
    if let Err(e) = &result {
        let _ = std::fs::remove_file(marker_path());
        log(&format!("update install failed: {e}"));
        set_tray_visible(app, true);
        publish(app, |s| {
            s.phase = "idle";
            s.error = Some(format!("Couldn't install v{}: {e}", update.version));
        });
    }
    result
}

/// Written just before the installer runs and read by the next start, so Hotbar knows it was just updated
/// (it comes back through the autostart task or the installer, neither of which can pass that on).
#[derive(Serialize, Deserialize)]
struct Marker {
    from: String,
    to: String,
}

fn marker_path() -> PathBuf {
    crate::config::path().with_file_name("updating.json")
}

/// At start: was an update just installed? Records the outcome in the status (shown in Settings →
/// Updates) and says whether to open the menu to report it, i.e. `updates.openAfterUpdate` for a
/// recent update.
pub fn after_restart(app: &AppHandle) -> bool {
    let path = marker_path();
    let Ok(text) = std::fs::read_to_string(&path) else { return false };
    let recent = std::fs::metadata(&path).and_then(|m| m.modified()).is_ok_and(|t| t.elapsed().is_ok_and(|e| e < REOPEN_WITHIN));
    let _ = std::fs::remove_file(&path);
    let Ok(m) = serde_json::from_str::<Marker>(&text) else { return false };
    match outcome(&m, &app.package_info().version.to_string()) {
        Ok(()) => {
            log(&format!("update: now running v{} (updated from v{})", m.to, m.from));
            publish(app, |s| s.updated_from = Some(m.from));
        }
        Err(e) => {
            log(&format!("update: {e}"));
            publish(app, |s| s.error = Some(e));
        }
    }
    recent && app.state::<crate::AppState>().store().cfg.updates.open_after_update
}

fn outcome(m: &Marker, current: &str) -> Result<(), String> {
    if m.to == current {
        Ok(())
    } else {
        Err(format!("The update to v{} didn't finish: still running v{current}.", m.to))
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

    #[test]
    fn progress_percent_is_clamped_and_needs_a_size() {
        assert_eq!(percent(0, Some(200)), Some(0));
        assert_eq!(percent(100, Some(200)), Some(50));
        assert_eq!(percent(300, Some(200)), Some(100));
        assert_eq!(percent(100, None), None);
        assert_eq!(percent(100, Some(0)), None);
    }

    fn status(phase: &'static str, available: Option<&str>, downloaded: u64, total: Option<u64>) -> Status {
        let available = available.map(|v| Release { version: v.into(), notes: None, date: None });
        Status { current: "0.1.5".into(), available, phase, checked_at: None, error: None, downloaded, total, updated_from: None }
    }

    #[test]
    fn a_restart_on_the_marker_version_counts_as_updated() {
        let m = Marker { from: "0.1.7".into(), to: "0.1.8".into() };
        assert!(outcome(&m, "0.1.8").is_ok());
        assert_eq!(outcome(&m, "0.1.7").unwrap_err(), "The update to v0.1.8 didn't finish: still running v0.1.7.");
    }

    #[test]
    fn tray_item_follows_the_status() {
        assert_eq!(tray_item(&status("idle", None, 0, None)), (CHECK_LABEL.into(), true));
        assert_eq!(tray_item(&status("checking", Some("0.1.6"), 0, None)), ("Checking for updates…".into(), false));
        assert_eq!(tray_item(&status("idle", Some("0.1.6"), 0, None)), ("Install update v0.1.6".into(), true));
        assert_eq!(tray_item(&status("downloading", Some("0.1.6"), 50, Some(200))), ("Downloading v0.1.6… 25%".into(), false));
        assert_eq!(tray_item(&status("downloading", Some("0.1.6"), 50, None)), ("Downloading v0.1.6…".into(), false));
    }
}
