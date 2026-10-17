//! Turning a project + mode into a Windows Terminal window running Claude Code.

use crate::config::{Config, Project};
use crate::winutil;
use serde::Serialize;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

/// What the overlay asked for: Enter = claude, ⇧Enter = continue, Alt+Enter = resume, Ctrl+Enter = shell.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Claude,
    Continue,
    Resume,
    Shell,
}

impl Mode {
    pub fn parse(s: &str) -> Self {
        match s {
            "continue" => Mode::Continue,
            "resume" => Mode::Resume,
            "shell" => Mode::Shell,
            _ => Mode::Claude,
        }
    }
}

/// Drive roots keep their backslash ("D:\"), everything else loses trailing ones.
pub fn normalize_dir(p: &str) -> String {
    let p = p.trim().trim_matches('"');
    let b = p.as_bytes();
    if b.len() >= 2 && b[1] == b':' && (b.len() == 2 || (b.len() == 3 && matches!(b[2], b'\\' | b'/'))) {
        return format!("{}:\\", &p[..1]);
    }
    p.trim_end_matches(['\\', '/']).to_string()
}

/// Quote one argument by the CommandLineToArgvW rules (backslashes before a quote are doubled).
fn quote(a: &str) -> String {
    if !a.is_empty() && !a.contains([' ', '\t', '"']) {
        return a.to_string();
    }
    let mut out = String::from("\"");
    let mut slashes = 0;
    for c in a.chars() {
        match c {
            '\\' => slashes += 1,
            '"' => {
                out.push_str(&"\\".repeat(slashes * 2 + 1));
                out.push('"');
                slashes = 0;
            }
            _ => {
                out.push_str(&"\\".repeat(slashes));
                out.push(c);
                slashes = 0;
            }
        }
    }
    out.push_str(&"\\".repeat(slashes * 2));
    out.push('"');
    out
}

/// The PowerShell command to run in the new tab, or None for a plain shell.
fn shell_command(p: &Project, mode: Mode) -> Option<String> {
    if mode == Mode::Shell {
        return None;
    }
    let mut claude = String::from("claude");
    if !p.flags.trim().is_empty() {
        claude.push(' ');
        claude.push_str(p.flags.trim());
    }
    match mode {
        Mode::Continue => claude.push_str(" --continue"),
        Mode::Resume => claude.push_str(" --resume"),
        _ => {}
    }
    Some(if p.pre.trim().is_empty() { claude } else { format!("{}; {}", p.pre.trim(), claude) })
}

pub fn title_for(p: &Project, mode: Mode) -> String {
    if mode == Mode::Shell { format!("{} (shell)", p.name) } else { p.name.clone() }
}

/// `#rgb`, `#rrggbb` or `#rrggbbaa`: anything else could smuggle extra arguments into the wt command line.
pub fn is_hex_color(c: &str) -> bool {
    c.strip_prefix('#').is_some_and(|h| matches!(h.len(), 3 | 4 | 6 | 8) && h.chars().all(|ch| ch.is_ascii_hexdigit()))
}

/// wt splits its command line on a bare `;` (even inside quotes); `\;` is a literal one.
fn wt_escape(a: &str) -> String {
    a.replace(';', "\\;")
}

/// Each project gets its own named wt window, so a second session opens as a tab beside the first.
/// The prefix keeps an all-digit name from being read as a window ID.
fn window_name(p: &Project) -> String {
    let n: String = p.name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    format!("hotbar-{n}")
}

/// Arguments for wt.exe, already quoted.
pub fn wt_args(cfg: &Config, p: &Project, mode: Mode) -> String {
    let dir = normalize_dir(&p.path);
    let mut a: Vec<String> = vec!["-w".into(), if cfg.launch.same_window { window_name(p) } else { "new".into() }];
    match cfg.launch.window.as_str() {
        "fullscreen" => a.push("--fullscreen".into()),
        "maximized" => a.push("--maximized".into()),
        _ => {}
    }
    a.push("new-tab".into());
    a.extend(["--title".into(), quote(&wt_escape(&title_for(p, mode))), "--suppressApplicationTitle".into()]);
    if is_hex_color(&p.color) {
        a.extend(["--tabColor".into(), p.color.clone()]);
    }
    a.extend(["-d".into(), quote(&wt_escape(&dir)), shell_exe(cfg).into(), "-NoExit".into()]);
    if let Some(cmd) = shell_command(p, mode) {
        a.extend(["-Command".into(), quote(&wt_escape(&cmd))]);
    }
    a.join(" ")
}

fn shell_exe(cfg: &Config) -> &'static str {
    if cfg.launch.shell == "pwsh" { "pwsh" } else { "powershell" }
}

/// Claude Code marks the processes it spawns (CLAUDECODE, CLAUDE_CODE_CHILD_SESSION, session id,
/// IPC pipe…). If Hotbar was started from inside a Claude session, passing those on would make the
/// new `claude` run as a sub-session (e.g. transcript saving off). Keep only CLAUDE* variables the
/// user set persistently in Windows, plus CLAUDE_CONFIG_DIR, which status() reads too.
fn inherited_claude_vars() -> Vec<std::ffi::OsString> {
    std::env::vars_os()
        .map(|(k, _)| k)
        .filter(|k| {
            let name = k.to_string_lossy().to_ascii_uppercase();
            name.starts_with("CLAUDE") && name != "CLAUDE_CONFIG_DIR" && !winutil::is_persistent_env_var(&name)
        })
        .collect()
}

/// wt.exe resolved through PATH only, never the project folder (a cloned repo could plant one).
fn wt_exe() -> PathBuf {
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d.is_absolute())
        .map(|d| d.join("wt.exe"))
        .find(|f| f.exists())
        .unwrap_or_else(|| PathBuf::from("wt.exe"))
}

fn clean_env(c: &mut Command) -> &mut Command {
    for k in inherited_claude_vars() {
        c.env_remove(k);
    }
    c
}

pub fn launch(cfg: &Config, p: &Project, mode: Mode, force_new: bool) -> Result<String, String> {
    let dir = normalize_dir(&p.path);
    if !Path::new(&dir).is_dir() {
        return Err(format!("Folder not found: {dir}"));
    }
    if cfg.launch.focus_existing && !force_new {
        if let Some(hwnd) = winutil::find_terminal(&title_for(p, mode)) {
            winutil::force_foreground(hwnd);
            return Ok(format!("Focused the open {} window", p.name));
        }
    }
    let args = wt_args(cfg, p, mode);
    let wt = wt_exe();
    if winutil::is_elevated() {
        // Never hand admin rights to a Claude session: no elevated console fallback either.
        return winutil::shell_execute_unelevated(&wt.to_string_lossy(), &args, &dir)
            .map(|()| format!("Opened {}", p.name))
            .map_err(|e| format!("Couldn't open {} without admin rights: {e}", p.name));
    }
    // wt.exe hands its environment to the new tab, so clean it here
    let spawned = clean_env(Command::new(&wt).raw_arg(&args).current_dir(&dir)).spawn().map(|_| ()).map_err(|e| e.to_string());
    match spawned {
        Ok(()) => Ok(format!("Opened {}", p.name)),
        Err(wt_err) => {
            // No Windows Terminal: plain console window instead
            let mut c = Command::new(format!("{}.exe", shell_exe(cfg)));
            clean_env(&mut c).arg("-NoExit");
            // PowerShell treats curly single quotes as quotes too: double every kind
            let title: String = title_for(p, mode)
                .chars()
                .flat_map(|c| if matches!(c, '\'' | '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}') { vec![c, c] } else { vec![c] })
                .collect();
            let set_title = format!("$Host.UI.RawUI.WindowTitle='{title}'");
            match shell_command(p, mode) {
                Some(cmd) => c.args(["-Command", &format!("{set_title}; {cmd}")]),
                None => c.args(["-Command", &set_title]),
            };
            c.current_dir(&dir)
                .creation_flags(CREATE_NEW_CONSOLE)
                .spawn()
                .map(|_| format!("Opened {} (wt.exe failed: {wt_err})", p.name))
                .map_err(|e| format!("Couldn't start a terminal: {e}"))
        }
    }
}

#[derive(Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub exists: bool,
    /// Unix seconds of the newest Claude Code transcript for this folder
    pub last_session: Option<u64>,
    pub sessions: u32,
    pub branch: Option<String>,
    pub running: bool,
}

fn claude_projects_dir() -> PathBuf {
    if let Ok(d) = std::env::var("CLAUDE_CONFIG_DIR") {
        return PathBuf::from(d).join("projects");
    }
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    PathBuf::from(home).join(".claude").join("projects")
}

/// Claude Code stores transcripts in a folder named after the path with every
/// non-alphanumeric character replaced by '-' (D:\CC Figure → D--CC-Figure).
fn transcript_dir(path: &str) -> PathBuf {
    let enc: String = normalize_dir(path).chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    claude_projects_dir().join(enc)
}

fn git_branch(dir: &str) -> Option<String> {
    let head = std::fs::read_to_string(Path::new(dir).join(".git").join("HEAD")).ok()?;
    let head = head.trim();
    Some(match head.strip_prefix("ref: refs/heads/") {
        Some(b) => b.to_string(),
        None => head.chars().take(7).collect(),
    })
}

pub fn status(p: &Project) -> Status {
    let dir = normalize_dir(&p.path);
    let mut st = Status { exists: Path::new(&dir).is_dir(), ..Default::default() };
    if let Ok(entries) = std::fs::read_dir(transcript_dir(&dir)) {
        for e in entries.flatten() {
            if e.path().extension().is_some_and(|x| x == "jsonl") {
                st.sessions += 1;
                if let Some(t) = e.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()) {
                    st.last_session = st.last_session.max(Some(t.as_secs()));
                }
            }
        }
    }
    st.branch = git_branch(&dir);
    st.running = winutil::find_terminal(&p.name).is_some();
    st
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting_follows_argv_rules() {
        assert_eq!(quote("D:\\CV-Web"), "D:\\CV-Web");
        assert_eq!(quote("D:\\CC Figure"), "\"D:\\CC Figure\"");
        assert_eq!(quote("a \"b\""), "\"a \\\"b\\\"\"");
        assert_eq!(quote("x y\\"), "\"x y\\\\\"");
    }

    #[test]
    fn dirs_normalize() {
        assert_eq!(normalize_dir("D:"), "D:\\");
        assert_eq!(normalize_dir("D:\\"), "D:\\");
        assert_eq!(normalize_dir("D:\\CC Figure\\"), "D:\\CC Figure");
        assert_eq!(normalize_dir("C:x"), "C:x");
    }

    #[test]
    fn colors_and_semicolons_cant_inject_wt_arguments() {
        assert!(is_hex_color("#e0563b") && is_hex_color("#f00"));
        assert!(!is_hex_color("#f00 ; new-tab calc") && !is_hex_color("red"));
        let cfg = Config::default();
        let mut p = cfg.projects[0].clone();
        p.color = "#f00 ; new-tab calc".into();
        p.name = "a;b".into();
        let a = wt_args(&cfg, &p, Mode::Claude);
        assert!(!a.contains("--tabColor"));
        assert!(a.contains("--title a\\;b"));
    }

    #[test]
    fn transcript_dir_matches_claude_naming() {
        assert!(transcript_dir("D:\\CC Figure").ends_with("D--CC-Figure"));
        assert!(transcript_dir("D:\\Hitachi-Live").ends_with("D--Hitachi-Live"));
    }

    #[test]
    fn claude_session_markers_are_stripped() {
        std::env::set_var("CLAUDE_CODE_CHILD_SESSION", "1");
        std::env::set_var("CLAUDECODE", "1");
        let vars: Vec<String> = inherited_claude_vars().iter().map(|v| v.to_string_lossy().into_owned()).collect();
        assert!(vars.contains(&"CLAUDE_CODE_CHILD_SESSION".to_string()));
        assert!(vars.contains(&"CLAUDECODE".to_string()));
        assert!(!vars.iter().any(|v| v == "PATH"));
    }

    #[test]
    fn wt_args_escape_semicolons_and_quote_paths() {
        let cfg = Config::default();
        let mut p = cfg.projects[5].clone(); // CC Figure
        p.pre = ".venv\\Scripts\\Activate.ps1".into();
        let a = wt_args(&cfg, &p, Mode::Continue);
        assert!(a.starts_with("-w hotbar-CC-Figure --maximized new-tab --title \"CC Figure\" --suppressApplicationTitle"));
        assert!(a.contains("-d \"D:\\CC Figure\" powershell -NoExit"));
        assert!(a.ends_with("-Command \".venv\\Scripts\\Activate.ps1\\; claude --continue\""));
    }
}
