//! `%APPDATA%\Hotbar\config.json` — loaded on start, re-read whenever its mtime changes.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, time::SystemTime};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub version: u32,
    pub trigger: Trigger,
    pub launch: Launch,
    /// Look & motion settings. Only the overlay reads these, so Rust keeps them as raw JSON.
    pub ui: Value,
    pub projects: Vec<Project>,
    pub other: Other,
    pub updates: Updates,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Trigger {
    #[serde(alias = "doubleTapRightCtrl")]
    pub double_tap: bool,
    /// Which Ctrl to double-tap: "leftCtrl" | "rightCtrl"
    pub key: String,
    /// Max gap between the two taps.
    pub tap_window_ms: u32,
    /// Ctrl Ctrl + digit within `chord_ms` opens that project without painting the overlay.
    pub blind_chord: bool,
    pub chord_ms: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Launch {
    /// "maximized" | "fullscreen" | "normal"
    pub window: String,
    pub focus_existing: bool,
    /// A project's second session opens as a tab in its first session's window, not a new window
    pub same_window: bool,
    /// Every project opens as a tab in one shared window (takes precedence over `same_window`)
    pub one_window: bool,
    /// "powershell" (Windows PowerShell 5.1) or "pwsh" (PowerShell 7)
    pub shell: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub color: String,
    /// What plain Enter does: "claude" | "continue" | "shell"
    pub enter: String,
    /// Optional PowerShell run before claude, e.g. `.venv\Scripts\Activate.ps1`
    pub pre: String,
    /// Extra claude flags, e.g. `--model opus`
    pub flags: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Other {
    pub path: String,
    /// "shell" = plain PowerShell there · "pick" = folder picker, then claude
    pub action: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Updates {
    /// Check 30 s after start and then daily (Settings → Updates and the tray check either way)
    pub auto_check: bool,
    /// After installing an update, Hotbar comes back showing Settings → Updates (instead of silently in the tray)
    pub open_after_update: bool,
}

impl Default for Updates {
    fn default() -> Self {
        Self { auto_check: true, open_after_update: true }
    }
}
impl Default for Trigger {
    fn default() -> Self {
        Self { double_tap: true, key: "leftCtrl".into(), tap_window_ms: 350, blind_chord: true, chord_ms: 200 }
    }
}
impl Default for Launch {
    fn default() -> Self {
        Self { window: "maximized".into(), focus_existing: true, same_window: true, one_window: false, shell: "powershell".into() }
    }
}
impl Default for Other {
    fn default() -> Self {
        Self { path: "D:\\".into(), action: "shell".into() }
    }
}

fn project(name: &str, path: &str, color: &str) -> Project {
    Project { name: name.into(), path: path.into(), color: color.into(), enter: "claude".into(), ..Default::default() }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            trigger: Trigger::default(),
            launch: Launch::default(),
            ui: json!({
                "layout": "blockcraft", "theme": "clay", "opacity": 0.62, "backdrop": "blur",
                "texture": "holo", "depth": "slab", "textureStrength": 0.6, "buttonTilt": true,
                "panelTilt": true, "openAnim": "drop", "selectAnim": "press", "sound": true, "animations": true
            }),
            projects: vec![
                project("RailPair", "D:\\Hitachi-Live", "#e0563b"),
                project("Hitachi CV", "D:\\HitachiCV", "#d6334f"),
                project("PingPair", "D:\\PingPair-PrivDev", "#3fa7d6"),
                project("My Web CV", "D:\\CV-Web", "#6cbf4a"),
                project("AI GPU", "D:\\AI-GPU", "#76b900"),
                project("CC Figure", "D:\\CC Figure", "#a77bf0"),
            ],
            other: Other::default(),
            updates: Updates::default(),
        }
    }
}

pub fn path() -> PathBuf {
    if let Ok(p) = std::env::var("HOTBAR_CONFIG") {
        return PathBuf::from(p);
    }
    let base = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."));
    base.join("Hotbar").join("config.json")
}

fn mtime() -> Option<SystemTime> {
    fs::metadata(path()).and_then(|m| m.modified()).ok()
}

pub struct Store {
    pub cfg: Config,
    loaded_at: Option<SystemTime>,
}

impl Store {
    /// Load the config, writing the defaults on first run.
    pub fn open() -> (Self, Option<String>) {
        if !path().exists() {
            let cfg = Config::default();
            let _ = write(&cfg);
            return (Self { cfg, loaded_at: mtime() }, None);
        }
        match read() {
            Ok(cfg) => (Self { cfg, loaded_at: mtime() }, None),
            // loaded_at stays None, so every refresh re-reads the file and reports the error until it's fixed
            Err(e) => (Self { cfg: Config::default(), loaded_at: None }, Some(e)),
        }
    }

    /// Re-read the file if it changed on disk. On a parse error the last good config stays in use,
    /// and the error is returned on every call until the file parses again.
    pub fn refresh(&mut self) -> Result<bool, String> {
        let now = mtime();
        if now == self.loaded_at {
            return Ok(false);
        }
        self.cfg = read()?;
        self.loaded_at = now;
        Ok(true)
    }

    /// Save `cfg`. If the file on disk doesn't parse (a hand edit gone wrong), it is kept as
    /// config.broken.json rather than silently overwritten.
    pub fn save(&mut self, cfg: Config) -> Result<(), String> {
        let p = path();
        if p.exists() && read().is_err() {
            let backup = p.with_file_name("config.broken.json");
            fs::copy(&p, &backup).map_err(|e| format!("Can't back up the broken config: {e}"))?;
            crate::log(&format!("unparsable config.json kept as {}", backup.display()));
        }
        write(&cfg)?;
        self.cfg = cfg;
        self.loaded_at = mtime();
        Ok(())
    }
}

fn read() -> Result<Config, String> {
    let text = fs::read_to_string(path()).map_err(|e| format!("Can't read config: {e}"))?;
    serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| format!("config.json: {e}"))
}

fn write(cfg: &Config) -> Result<(), String> {
    let p = path();
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    // write, flush to disk, then rename, so a crash mid-save never leaves a half-written file
    let tmp = p.with_extension("json.tmp");
    let written = fs::File::create(&tmp).and_then(|mut f| {
        use std::io::Write;
        f.write_all(text.as_bytes())?;
        f.sync_all()
    });
    let result = written.and_then(|_| fs::rename(&tmp, &p));
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result.map_err(|e| format!("Can't save config: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broken_config_keeps_reporting_and_is_backed_up_before_a_save() {
        let dir = std::env::temp_dir().join(format!("hotbar-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("config.json");
        fs::write(&file, "{ not json").unwrap();
        std::env::set_var("HOTBAR_CONFIG", &file);

        let (mut store, err) = Store::open();
        assert!(err.is_some(), "the parse error is reported at start");
        assert!(store.refresh().is_err(), "and again on the next refresh");
        assert!(store.refresh().is_err(), "until the file is fixed");

        store.save(Config::default()).unwrap();
        assert_eq!(fs::read_to_string(dir.join("config.broken.json")).unwrap(), "{ not json");
        assert_eq!(store.refresh(), Ok(false), "the saved file parses");

        std::env::remove_var("HOTBAR_CONFIG");
        let _ = fs::remove_dir_all(&dir);
    }
}