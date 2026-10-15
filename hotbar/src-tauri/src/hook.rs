//! Low-level keyboard hook: double-tapping Ctrl (left by default) summons the overlay, and a digit pressed
//! right after (the "blind chord") opens that project without waiting for the overlay to paint.
//!
//! A plain RegisterHotKey can't express "a modifier tapped twice", hence WH_KEYBOARD_LL.
//! The hook proc runs on the thread that installed it, so its state can live in a thread_local.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::Threading::{GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_TIME_CRITICAL};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

pub enum Event {
    Summon,
    Digit(u8),
}

pub static ENABLED: AtomicBool = AtomicBool::new(true);
pub static TRIGGER_VK: AtomicU32 = AtomicU32::new(VK_LCONTROL);
pub static TAP_MS: AtomicU32 = AtomicU32::new(350);
pub static CHORD_ON: AtomicBool = AtomicBool::new(true);
pub static CHORD_MS: AtomicU32 = AtomicU32::new(200);
/// Set by the app; the chord only applies when the double-tap is *opening* the overlay.
pub static OVERLAY_VISIBLE: AtomicBool = AtomicBool::new(false);
/// Set when the settings change; the hook thread then forgets any half-finished tap sequence.
pub static RESET: AtomicBool = AtomicBool::new(false);
/// GetTickCount() when the overlay was last hidden: the same clock as KBDLLHOOKSTRUCT::time.
static HIDDEN_TICK: AtomicU32 = AtomicU32::new(0);
/// A double-tap this soon after a hide is the one the page itself used to close the menu.
const AFTER_HIDE_MS: u32 = 300;

pub fn mark_hidden() {
    HIDDEN_TICK.store(unsafe { GetTickCount() }.max(1), Relaxed);
}

static TX: OnceLock<Sender<Event>> = OnceLock::new();

pub const VK_LCONTROL: u32 = 0xA2;
pub const VK_RCONTROL: u32 = 0xA3;
/// A press longer than this is a hold (e.g. Ctrl+click), not a tap.
const TAP_MAX_HOLD_MS: u32 = 250;

#[derive(Default)]
struct Taps {
    down: bool,
    down_at: u32,
    /// another key was pressed while the trigger Ctrl was held
    dirty: bool,
    last_tap: Option<u32>,
    chord_from: Option<u32>,
    swallow_up: Option<u32>,
}

thread_local! {
    static STATE: RefCell<Taps> = RefCell::new(Taps::default());
}

/// Synthetic key presses (AutoHotkey, SendInput…) are ignored unless HOTBAR_ACCEPT_INJECTED=1,
/// which exists for scripted end-to-end tests.
static ACCEPT_INJECTED: AtomicBool = AtomicBool::new(false);

pub fn start(tx: Sender<Event>) {
    let _ = TX.set(tx);
    ACCEPT_INJECTED.store(std::env::var("HOTBAR_ACCEPT_INJECTED").is_ok_and(|v| v == "1"), Relaxed);
    let _ = std::thread::Builder::new().name("hotbar-keyboard-hook".into()).spawn(|| unsafe {
        // Windows skips (and eventually drops) a low-level hook that answers late, so this thread
        // must never wait behind anything else.
        let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);
        let module = GetModuleHandleW(None).ok().map(|m| HINSTANCE(m.0));
        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), module, 0) {
            Ok(_) => crate::log(&format!("keyboard hook installed (synthetic keys accepted: {})", ACCEPT_INJECTED.load(Relaxed))),
            Err(e) => return crate::log(&format!("keyboard hook FAILED: {e}")),
        }
        // LL hooks are called through this thread's message loop.
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {}
        crate::log("keyboard hook message loop ended");
    });
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && ENABLED.load(Relaxed) {
        let k = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        if ACCEPT_INJECTED.load(Relaxed) {
            // test mode only: file I/O has no place in a hook otherwise
            crate::log(&format!("key msg={:#x} vk={:#x} flags={:#x}", wparam.0, k.vkCode, k.flags.0));
        }
        let msg = wparam.0 as u32;
        let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
        let injected = k.flags.0 & LLKHF_INJECTED.0 != 0;
        if (!injected || ACCEPT_INJECTED.load(Relaxed)) && STATE.with(|s| on_key(&mut s.borrow_mut(), k.vkCode, k.time, down, up)) {
            return LRESULT(1); // swallow
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

fn send(ev: Event) {
    if let Some(tx) = TX.get() {
        let _ = tx.send(ev);
    }
}

fn digit(vk: u32) -> Option<u8> {
    match vk {
        0x30..=0x39 => Some((vk - 0x30) as u8),
        0x60..=0x69 => Some((vk - 0x60) as u8), // numpad
        _ => None,
    }
}

/// Returns true when the key should be swallowed.
fn on_key(s: &mut Taps, vk: u32, t: u32, down: bool, up: bool) -> bool {
    if RESET.swap(false, Relaxed) {
        *s = Taps::default();
    }
    if up && s.swallow_up == Some(vk) {
        s.swallow_up = None;
        return true;
    }
    if down {
        if let Some(from) = s.chord_from.take() {
            if t.wrapping_sub(from) <= CHORD_MS.load(Relaxed) {
                if let Some(d) = digit(vk) {
                    s.swallow_up = Some(vk);
                    send(Event::Digit(d));
                    return true;
                }
            }
        }
    }
    if vk == TRIGGER_VK.load(Relaxed) {
        if down && !s.down {
            // ignore auto-repeat: only the first key-down of a press counts
            s.down = true;
            s.down_at = t;
            s.dirty = false;
        } else if up {
            s.down = false;
            let tap = !s.dirty && t.wrapping_sub(s.down_at) <= TAP_MAX_HOLD_MS;
            match s.last_tap {
                Some(prev) if tap && t.wrapping_sub(prev) <= TAP_MS.load(Relaxed) => {
                    s.last_tap = None;
                    let hidden = HIDDEN_TICK.load(Relaxed);
                    if hidden != 0 && t.wrapping_sub(hidden) < AFTER_HIDE_MS {
                        return false;
                    }
                    if CHORD_ON.load(Relaxed) && !OVERLAY_VISIBLE.load(Relaxed) {
                        s.chord_from = Some(t);
                    }
                    send(Event::Summon);
                }
                _ => s.last_tap = tap.then_some(t),
            }
        }
    } else if down {
        // any other key breaks a tap sequence (so Ctrl+C, Ctrl+V… never summon)
        s.dirty = true;
        s.last_tap = None;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(s: &mut Taps, vk: u32, t: u32, down: bool) -> bool {
        on_key(s, vk, t, down, !down)
    }

    #[test]
    fn double_tap_left_ctrl_summons_and_shortcuts_dont() {
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = TX.set(tx);
        let s = &mut Taps::default();
        // Ctrl+C then Ctrl+V: never a summon
        key(s, VK_LCONTROL, 0, true);
        key(s, 0x43, 20, true);
        key(s, 0x43, 60, false);
        key(s, VK_LCONTROL, 80, false);
        key(s, VK_LCONTROL, 150, true);
        key(s, 0x56, 170, true);
        key(s, 0x56, 200, false);
        key(s, VK_LCONTROL, 220, false);
        assert!(rx.try_recv().is_err());
        // Right Ctrl taps don't count while Left Ctrl is the trigger
        for (t, down) in [(1000, true), (1050, false), (1100, true), (1150, false)] {
            key(s, VK_RCONTROL, t, down);
        }
        assert!(rx.try_recv().is_err());
        // Left Ctrl, Left Ctrl, then "3" quickly: summon, and the digit is swallowed
        key(s, VK_LCONTROL, 2000, true);
        key(s, VK_LCONTROL, 2060, false);
        key(s, VK_LCONTROL, 2150, true);
        key(s, VK_LCONTROL, 2210, false);
        assert!(matches!(rx.try_recv(), Ok(Event::Summon)));
        assert!(key(s, 0x33, 2300, true), "digit swallowed");
        assert!(key(s, 0x33, 2350, false), "and its key-up");
        assert!(matches!(rx.try_recv(), Ok(Event::Digit(3))));
    }
}
