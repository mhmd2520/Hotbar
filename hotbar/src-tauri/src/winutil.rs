//! Win32 helpers: monitor geometry, foreground activation, window lookup and
//! launching unelevated from an elevated process.

use windows::core::{Interface, BSTR, HSTRING};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IDispatch, IServiceProvider, CLSCTX_LOCAL_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::Shell::{
    IShellBrowser, IShellDispatch2, IShellFolderViewDual, IShellView, IShellWindows, ShellWindows,
    CSIDL_DESKTOP, SID_STopLevelBrowser, SVGIO_BACKGROUND, SWC_DESKTOP, SWFO_NEEDDISPATCH,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, FindWindowW, GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId, IsIconic,
    SetForegroundWindow, ShowWindow, SW_RESTORE,
};

/// Work area (x, y, width, height) of the monitor under the mouse cursor, in physical pixels.
pub fn cursor_work_area() -> Option<(i32, i32, u32, u32)> {
    unsafe {
        let mut pt = POINT::default();
        GetCursorPos(&mut pt).ok()?;
        let mon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if !GetMonitorInfoW(mon, &mut mi).as_bool() {
            return None;
        }
        let r = mi.rcWork;
        Some((r.left, r.top, (r.right - r.left) as u32, (r.bottom - r.top) as u32))
    }
}

/// Bring `hwnd` to the foreground even when another app currently owns it.
/// Windows normally refuses SetForegroundWindow from a background process, so we briefly
/// attach our input queue to the foreground thread's.
pub fn force_foreground(hwnd: HWND) {
    unsafe {
        let fg = GetForegroundWindow();
        let fg_thread = GetWindowThreadProcessId(fg, None);
        let me = GetCurrentThreadId();
        let attached = fg_thread != 0 && fg_thread != me && AttachThreadInput(me, fg_thread, true).as_bool();
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = BringWindowToTop(hwnd);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(Some(hwnd));
        if attached {
            let _ = AttachThreadInput(me, fg_thread, false);
        }
    }
}

/// True if the foreground window belongs to this process (the overlay or one of its child windows).
pub fn foreground_is_ours() -> bool {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut pid));
        pid == std::process::id()
    }
}

/// A Windows Terminal window whose title is exactly `title`.
pub fn find_terminal(title: &str) -> Option<HWND> {
    unsafe { FindWindowW(&HSTRING::from("CASCADIA_HOSTING_WINDOW_CLASS"), &HSTRING::from(title)).ok() }
}

/// True if `name` is set in the user's or the machine's persistent environment (the registry),
/// as opposed to being inherited from whatever process started Hotbar.
pub fn is_persistent_env_var(name: &str) -> bool {
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_ANY};
    let value = HSTRING::from(name);
    [
        (HKEY_CURRENT_USER, HSTRING::from("Environment")),
        (HKEY_LOCAL_MACHINE, HSTRING::from("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")),
    ]
    .iter()
    .any(|(root, key)| unsafe { RegGetValueW(*root, key, &value, RRF_RT_ANY, None, None, None) == ERROR_SUCCESS })
}

/// Opt out of Windows 11 "efficiency mode" (EcoQoS). A tray app with a hidden window is a prime
/// candidate for it, and a throttled process answers the keyboard hook too late.
pub fn disable_power_throttling() {
    use windows::Win32::System::Threading::{
        GetCurrentProcess, ProcessPowerThrottling, SetProcessInformation, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
    };
    let state = PROCESS_POWER_THROTTLING_STATE {
        Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
        ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        StateMask: 0, // controlled, and off
    };
    unsafe {
        let _ = SetProcessInformation(
            GetCurrentProcess(),
            ProcessPowerThrottling,
            &state as *const _ as *const _,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

/// When Hotbar runs elevated, Windows (UIPI) drops messages sent to it by normal-integrity apps.
/// That includes the WM_COPYDATA a second `hotbar.exe` (Start menu / desktop shortcut) uses to say
/// "open the menu", so the shortcut silently did nothing. Let exactly that message through to the
/// single-instance plugin's window (class `<identifier>-sic`).
pub fn accept_second_instance_messages(identifier: &str) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{ChangeWindowMessageFilterEx, MSGFLT_ALLOW, WM_COPYDATA};
    unsafe {
        let Ok(hwnd) = FindWindowW(&HSTRING::from(format!("{identifier}-sic")), None) else { return false };
        ChangeWindowMessageFilterEx(hwnd, WM_COPYDATA, MSGFLT_ALLOW, None).is_ok()
    }
}

/// True if this process runs with an elevated (admin) token.
pub fn is_elevated() -> bool {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

/// Run `file args` in `dir` with the desktop shell's (unelevated) token.
/// Used when Hotbar itself runs elevated, so Claude sessions don't inherit admin rights.
/// See Raymond Chen, "How can I launch an unelevated process from my elevated process?"
pub fn shell_execute_unelevated(file: &str, args: &str, dir: &str) -> Result<(), String> {
    let (file, args, dir) = (file.to_owned(), args.to_owned(), dir.to_owned());
    // COM wants its own STA thread; keep it off the async runtime.
    std::thread::spawn(move || unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let res = (|| -> windows::core::Result<()> {
            let windows: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_LOCAL_SERVER)?;
            let loc = VARIANT::from(CSIDL_DESKTOP as i32);
            let root = VARIANT::default();
            let mut hwnd = 0i32;
            let desktop: IDispatch = windows.FindWindowSW(&loc, &root, SWC_DESKTOP, &mut hwnd, SWFO_NEEDDISPATCH)?;
            let sp: IServiceProvider = desktop.cast()?;
            let browser: IShellBrowser = sp.QueryService(&SID_STopLevelBrowser)?;
            let view: IShellView = browser.QueryActiveShellView()?;
            let bg: IDispatch = view.GetItemObject(SVGIO_BACKGROUND)?;
            let folder: IShellFolderViewDual = bg.cast()?;
            let shell: IShellDispatch2 = folder.Application()?.cast()?;
            shell.ShellExecute(
                &BSTR::from(file.as_str()),
                &VARIANT::from(BSTR::from(args.as_str())),
                &VARIANT::from(BSTR::from(dir.as_str())),
                &VARIANT::from(BSTR::from("open")),
                &VARIANT::from(1i32),
            )
        })();
        CoUninitialize();
        res.map_err(|e| format!("unelevated launch failed: {e}"))
    })
    .join()
    .map_err(|_| "unelevated launch thread panicked".to_string())?
}
