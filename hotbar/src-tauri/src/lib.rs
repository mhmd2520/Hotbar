mod config;
mod hook;
mod launch;
mod update;
mod winutil;

use config::{Config, Project};
use serde::Serialize;
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::{mpsc, Mutex, MutexGuard, OnceLock};
use std::time::Instant;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::window::{Effect, EffectsBuilder};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewWindow, WindowEvent};
use tauri_plugin_dialog::DialogExt;

struct AppState {
    store: Mutex<config::Store>,
    /// While the settings view is open, losing focus doesn't hide the window.
    pinned: AtomicBool,
}

impl AppState {
    /// The config store. A panic while it was held must not take every later command down with it.
    fn store(&self) -> MutexGuard<'_, config::Store> {
        self.store.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    config: Config,
    status: Vec<launch::Status>,
    config_path: String,
    elevated: bool,
    error: Option<String>,
    update: update::Status,
}

const TOOLTIP: &str = "Hotbar: double-tap Ctrl";

fn main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("main")
}

fn apply_runtime(app: &AppHandle, cfg: &Config) {
    hook::ENABLED.store(cfg.trigger.double_tap, Relaxed);
    let vk = if cfg.trigger.key == "rightCtrl" { hook::VK_RCONTROL } else { hook::VK_LCONTROL };
    hook::TRIGGER_VK.store(vk, Relaxed);
    hook::TAP_MS.store(cfg.trigger.tap_window_ms.clamp(150, 1000), Relaxed);
    hook::CHORD_ON.store(cfg.trigger.blind_chord, Relaxed);
    hook::CHORD_MS.store(cfg.trigger.chord_ms.clamp(80, 1000), Relaxed);
    hook::RESET.store(true, Relaxed);
    if let Some(w) = main_window(app) {
        let effect = match cfg.ui.get("backdrop").and_then(|v| v.as_str()) {
            Some("none") => None,
            Some("acrylic") => Some(Effect::Acrylic),
            _ => Some(Effect::Blur),
        };
        let _ = w.set_effects(effect.map(|e| EffectsBuilder::new().effect(e).build()));
    }
}

/// Re-read config.json if it changed; on a parse error keep the last good config and report it.
/// The lock is released before apply_runtime, which talks to the window.
fn refresh(app: &AppHandle, state: &AppState) -> Option<String> {
    let changed = {
        let mut store = state.store();
        store.refresh().map(|changed| changed.then(|| store.cfg.clone()))
    };
    match changed {
        Ok(Some(cfg)) => {
            apply_runtime(app, &cfg);
            None
        }
        Ok(None) => None,
        Err(e) => Some(e),
    }
}

/// Appends a line to %APPDATA%\Hotbar\hotbar.log (restarted once it passes 64 KB).
pub(crate) fn log(msg: &str) {
    use std::io::Write;
    let path = config::path().with_file_name("hotbar.log");
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > 64 * 1024) {
        let _ = std::fs::remove_file(&path);
    }
    let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{}.{:03} {msg}", ms / 1000, ms % 1000);
    }
}

/// Bumped on every summon/digit so a stale delayed show can tell it was superseded.
static SUMMON_GEN: AtomicU64 = AtomicU64::new(0);
/// Bumped on every show, so a focus check from an earlier show doesn't act on a newer one.
static SHOW_GEN: AtomicU64 = AtomicU64::new(0);
/// When the overlay was last shown (monotonic ms), for the focus grace period.
static SHOWN_AT: AtomicU64 = AtomicU64::new(0);
/// Focus taken back once already for this show.
static RETOOK_FOCUS: AtomicBool = AtomicBool::new(false);

/// Monotonic milliseconds since start (immune to clock changes).
fn now_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// Win the foreground first (another app owns it), then let Tauri put keyboard focus in the WebView.
fn focus_overlay(w: &WebviewWindow) {
    if let Ok(h) = w.hwnd() {
        winutil::force_foreground(windows::Win32::Foundation::HWND(h.0 as _));
    }
    let _ = w.set_focus();
}

fn show_overlay(app: &AppHandle, view: &str) {
    let Some(w) = main_window(app) else { return };
    let state = app.state::<AppState>();
    let error = refresh(app, &state);
    // cover the work area of the monitor the mouse is on (size first, so a move between monitors
    // with different scaling doesn't resize it with the old monitor's scale)
    if let Some((x, y, width, height)) = winutil::cursor_work_area() {
        let _ = w.set_size(PhysicalSize::new(width, height));
        let _ = w.set_position(PhysicalPosition::new(x, y));
        let _ = w.set_size(PhysicalSize::new(width, height));
    }
    let _ = w.emit("summon", json!({ "view": view, "error": error }));
    SHOW_GEN.fetch_add(1, Relaxed);
    SHOWN_AT.store(now_ms(), Relaxed);
    RETOOK_FOCUS.store(false, Relaxed);
    hook::OVERLAY_VISIBLE.store(true, Relaxed);
    let _ = w.show();
    focus_overlay(&w);
    log(&format!("show ({view}), foreground is ours: {}", winutil::foreground_is_ours()));
}

/// "Click outside / switch away hides the menu", without hiding it on the focus shuffles that
/// happen while it opens: focus moving into the WebView, or the app the user was in briefly
/// reclaiming the foreground (Windows' foreground lock). Called on the window's focus-lost event.
fn on_focus_lost(app: &AppHandle) {
    if app.state::<AppState>().pinned.load(Relaxed) {
        return;
    }
    let app = app.clone();
    let gen = SHOW_GEN.load(Relaxed);
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(120));
        let stale = || SHOW_GEN.load(Relaxed) != gen || !hook::OVERLAY_VISIBLE.load(Relaxed);
        if stale() || winutil::foreground_is_ours() || app.state::<AppState>().pinned.load(Relaxed) {
            return; // shown again / already hidden, or focus only moved inside Hotbar
        }
        let fresh = now_ms().saturating_sub(SHOWN_AT.load(Relaxed)) < 700;
        if fresh && !RETOOK_FOCUS.swap(true, Relaxed) {
            log("focus taken right after opening; taking it back");
            if let Some(w) = main_window(&app) {
                focus_overlay(&w);
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
            if stale() || winutil::foreground_is_ours() {
                return;
            }
        }
        log("focus lost to another app");
        hide_overlay(&app);
    });
}

fn hide_overlay(app: &AppHandle) {
    log("hide");
    if let Some(w) = main_window(app) {
        let _ = w.hide();
        let _ = w.emit("hidden", ());
    }
    hook::OVERLAY_VISIBLE.store(false, Relaxed);
    hook::mark_hidden();
    app.state::<AppState>().pinned.store(false, Relaxed);
}

fn toggle_overlay(app: &AppHandle) {
    let visible = main_window(app).and_then(|w| w.is_visible().ok()).unwrap_or(false);
    if visible {
        return hide_overlay(app);
    }
    if !hook::CHORD_ON.load(Relaxed) {
        return show_overlay(app, "menu");
    }
    // Blind chord: wait briefly before painting; a digit in that window opens the project
    // directly and the overlay (and its backdrop blur) never appears.
    let gen = SUMMON_GEN.fetch_add(1, Relaxed) + 1;
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(hook::CHORD_MS.load(Relaxed) as u64));
        if SUMMON_GEN.load(Relaxed) == gen {
            show_overlay(&app, "menu");
        }
    });
}

/// A blind-chord digit: open that project straight away. A digit with no project behind it
/// opens the menu instead, so the swallowed keystroke never just disappears.
fn chord_digit(app: &AppHandle, d: u8) {
    SUMMON_GEN.fetch_add(1, Relaxed); // cancel the pending show
    let cfg = app.state::<AppState>().store().cfg.clone();
    let n = cfg.projects.len();
    let index = match d as usize {
        0 => n,
        d if d <= n.min(9) => d - 1,
        _ => return show_overlay(app, "menu"),
    };
    let mode = cfg.projects.get(index).map(|p| p.enter.clone()).unwrap_or_default();
    let _ = open_index(app, &cfg, index, &mode, false);
}

/// Open project `index` (`projects.len()` = "Other…"). On failure the menu comes back with the reason.
fn open_index(app: &AppHandle, cfg: &Config, index: usize, mode: &str, force_new: bool) -> Result<String, String> {
    hide_overlay(app);
    let result = match cfg.projects.get(index) {
        // A placeholder with no folder yet (e.g. via the blind chord): Settings, to pick one
        Some(p) if p.path.trim().is_empty() => {
            show_overlay(app, "settings");
            return Ok(format!("{} has no folder yet", p.name));
        }
        Some(p) => launch::launch(cfg, p, launch::Mode::parse(mode), force_new),
        None if cfg.other.action == "pick" => {
            let start = launch::normalize_dir(&cfg.other.path);
            let Some(dir) = app.dialog().file().set_directory(&start).set_title("Open Claude in…").blocking_pick_folder()
            else {
                return Ok("Cancelled".into());
            };
            let path = dir.to_string();
            let name = std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or(path.clone());
            let p = Project { name, path, color: "#8d8d8d".into(), enter: "claude".into(), ..Default::default() };
            launch::launch(cfg, &p, launch::Mode::parse(mode), force_new)
        }
        None => {
            let p = Project { name: "Shell".into(), path: cfg.other.path.clone(), ..Default::default() };
            launch::launch(cfg, &p, launch::Mode::Shell, true)
        }
    };
    result.inspect_err(|e| {
        log(&format!("open failed: {e}"));
        show_overlay(app, "menu");
        let _ = app.emit("toast", e.clone());
    })
}

#[tauri::command]
async fn snapshot(app: AppHandle, state: State<'_, AppState>) -> Result<Snapshot, String> {
    let error = refresh(&app, &state);
    let cfg = state.store().cfg.clone();
    let tabs = winutil::terminal_tab_titles();
    let status = cfg.projects.iter().map(|p| launch::status(p, &tabs)).collect();
    Ok(Snapshot {
        config: cfg,
        status,
        config_path: config::path().display().to_string(),
        elevated: winutil::is_elevated(),
        error,
        update: update::status(&app),
    })
}

#[tauri::command]
async fn save_config(app: AppHandle, state: State<'_, AppState>, config: Config) -> Result<(), String> {
    let cfg = {
        let mut store = state.store();
        store.save(config)?;
        store.cfg.clone()
    };
    apply_runtime(&app, &cfg);
    Ok(())
}

/// `index` == projects.len() means "Other…".
#[tauri::command]
async fn open_project(
    app: AppHandle,
    state: State<'_, AppState>,
    index: usize,
    mode: String,
    force_new: bool,
) -> Result<String, String> {
    let cfg = state.store().cfg.clone();
    open_index(&app, &cfg, index, &mode, force_new)
}

#[tauri::command]
fn hide(app: AppHandle) {
    hide_overlay(&app);
}

#[tauri::command]
fn set_pinned(state: State<AppState>, pinned: bool) {
    state.pinned.store(pinned, Relaxed);
}

#[tauri::command]
async fn pick_folder(app: AppHandle, start: Option<String>) -> Option<String> {
    let state = app.state::<AppState>();
    state.pinned.store(true, Relaxed); // the dialog steals focus; don't hide the settings
    let mut d = app.dialog().file().set_title("Choose the project folder");
    if let Some(s) = start.filter(|s| !s.trim().is_empty()) {
        d = d.set_directory(launch::normalize_dir(&s));
    }
    let w = main_window(&app);
    if let Some(w) = &w {
        d = d.set_parent(w); // modal to the always-on-top overlay, so it can't open behind it
    }
    let picked = d.blocking_pick_folder().map(|p| p.to_string());
    if let Some(w) = w {
        let _ = w.set_focus();
    }
    picked
}

#[tauri::command]
fn open_config_file() {
    let _ = std::process::Command::new("notepad.exe").arg(config::path()).spawn();
}

pub fn run() {
    let (store, load_error) = config::Store::open();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let view = if argv.iter().any(|a| a == "--settings") { "settings" } else { "menu" };
            show_overlay(app, view);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState { store: Mutex::new(store), pinned: AtomicBool::new(false) })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            save_config,
            open_project,
            hide,
            set_pinned,
            pick_folder,
            open_config_file,
            update::update_status,
            update::check_update,
            update::install_update
        ])
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                hide_overlay(window.app_handle());
            }
            WindowEvent::Focused(false) => on_focus_lost(window.app_handle()),
            _ => {}
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            let cfg = app.state::<AppState>().store().cfg.clone();
            apply_runtime(&handle, &cfg);

            // keyboard hook → worker thread → overlay
            winutil::disable_power_throttling();
            let (tx, rx) = mpsc::channel();
            hook::start(tx);
            let worker = handle.clone();
            std::thread::spawn(move || {
                for ev in rx {
                    match ev {
                        hook::Event::Summon => {
                            log("summon");
                            toggle_overlay(&worker)
                        }
                        hook::Event::Digit(d) => {
                            log(&format!("chord digit {d}"));
                            chord_digit(&worker, d)
                        }
                    }
                }
            });

            let open = MenuItemBuilder::with_id("open", "Open Hotbar    (Ctrl ×2)").build(app)?;
            let settings = MenuItemBuilder::with_id("settings", "Settings…").build(app)?;
            let edit = MenuItemBuilder::with_id("edit", "Edit config.json").build(app)?;
            let check = MenuItemBuilder::with_id(update::MENU_ID, update::CHECK_LABEL).build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit Hotbar").build(app)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let menu = MenuBuilder::new(app).items(&[&open, &settings, &edit, &check, &sep, &quit]).build()?;
            TrayIconBuilder::with_id("hotbar")
                .icon(app.default_window_icon().cloned().expect("bundled icon"))
                .tooltip(TOOLTIP)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, ev| match ev.id().as_ref() {
                    "open" => show_overlay(app, "menu"),
                    "settings" => show_overlay(app, "settings"),
                    "edit" => open_config_file(),
                    update::MENU_ID => update::on_menu(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, ev| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                        show_overlay(tray.app_handle(), "menu");
                    }
                })
                .build(app)?;
            update::start(&handle, check);
            let updated = update::after_restart(&handle);

            // Autostart passes --background and stays silent in the tray. A start from the Start menu
            // or desktop icon opens the menu, so the click visibly does something. Right after an
            // update (with updates.openAfterUpdate) it opens the menu either way, saying it updated.
            // A broken config.json is reported whenever the menu opens (Store keeps returning the
            // parse error).
            let view = if updated {
                Some("updated")
            } else {
                (!std::env::args().any(|a| a == "--background")).then_some("menu")
            };
            if let Some(view) = view {
                let h = handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500)); // let the page load
                    show_overlay(&h, view);
                });
            }
            if winutil::is_elevated() {
                let ok = winutil::accept_second_instance_messages(&app.config().identifier);
                log(&format!("elevated; shortcut messages from normal apps allowed: {ok}"));
            }
            log(&format!("started, pid {}", std::process::id()));
            if let Some(e) = &load_error {
                log(&format!("config error: {e}"));
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Hotbar");
}
