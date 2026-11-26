//! Early startup helpers: file logging, WebView2 detection, native error dialogs.
//!
//! Portable / corporate machines often have no console (`windows_subsystem = "windows"`),
//! so silent exits are useless. Always write `<exe>/data/quicknotes.log` (portable) or
//! beside the exe / AppData, and show a MessageBox for fatal runtime gaps.

use crate::models::{config_dir, exe_dir, is_portable_mode};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

const WEBVIEW2_DOWNLOAD: &str =
    "https://developer.microsoft.com/microsoft-edge/webview2/";
const WEBVIEW2_BOOTSTRAPPER_NAME: &str = "MicrosoftEdgeWebview2Setup.exe";
const WEBVIEW2_CLIENT_GUID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

/// Resolve and create the log file path. Prefer portable `data/`, then AppData, then exe dir.
pub fn init_logging() -> PathBuf {
    let candidates: Vec<PathBuf> = {
        let mut v = Vec::new();
        if is_portable_mode() {
            v.push(config_dir().join("quicknotes.log"));
        }
        if let Some(dir) = exe_dir() {
            v.push(dir.join("data").join("quicknotes.log"));
            v.push(dir.join("quicknotes.log"));
        }
        if let Some(dir) = dirs::config_dir() {
            v.push(dir.join("QuickNotes").join("quicknotes.log"));
        }
        v.push(PathBuf::from("quicknotes.log"));
        v
    };

    for path in &candidates {
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                continue;
            }
        }
        if OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .is_ok()
        {
            if let Ok(mut guard) = LOG_PATH.lock() {
                *guard = Some(path.clone());
            }
            log_line(&format!("=== QuickNotes start pid={} ===", std::process::id()));
            log_line(&format!("log_path={}", path.display()));
            if let Ok(exe) = std::env::current_exe() {
                log_line(&format!("exe={}", exe.display()));
            }
            log_line(&format!("portable={}", is_portable_mode()));
            return path.clone();
        }
    }

    PathBuf::from("quicknotes.log")
}

pub fn log_path() -> Option<PathBuf> {
    LOG_PATH.lock().ok().and_then(|g| g.clone())
}

pub fn log_line(message: &str) {
    let ts = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let line = format!("[{ts}] {message}\n");
    let _ = eprintln!("{message}");

    let path = LOG_PATH.lock().ok().and_then(|g| g.clone());
    let Some(path) = path else {
        return;
    };
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}

pub fn show_error(title: &str, message: &str) {
    log_line(&format!("ERROR: {title}: {message}"));
    #[cfg(windows)]
    {
        message_box(title, message);
    }
    #[cfg(not(windows))]
    {
        let _ = (title, message);
    }
}

/// Returns true when Evergreen WebView2 Runtime appears installed (HKLM/HKCU).
pub fn webview2_runtime_available() -> bool {
    #[cfg(windows)]
    {
        webview2_runtime_available_windows()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// If a bundled bootstrapper sits next to the exe, launch it (interactive).
/// Returns true if the process was started successfully.
pub fn try_launch_webview2_bootstrapper() -> bool {
    let Some(dir) = exe_dir() else {
        return false;
    };
    let setup = dir.join(WEBVIEW2_BOOTSTRAPPER_NAME);
    if !setup.is_file() {
        log_line(&format!(
            "WebView2 bootstrapper not found at {}",
            setup.display()
        ));
        return false;
    }
    log_line(&format!("Launching WebView2 bootstrapper: {}", setup.display()));
    match std::process::Command::new(&setup).spawn() {
        Ok(_) => {
            log_line("WebView2 bootstrapper started; user should finish install and relaunch");
            true
        }
        Err(e) => {
            log_line(&format!("Failed to launch WebView2 bootstrapper: {e}"));
            false
        }
    }
}

/// Pre-flight before Tauri window creation. Shows a clear MessageBox when Runtime looks missing.
///
/// Returns `false` only when the bundled bootstrapper was launched (user should finish install
/// and relaunch). Otherwise returns `true` so a false-negative registry check cannot brick startup —
/// window creation will MessageBox again if WebView2 truly fails.
pub fn ensure_webview2_or_explain() -> bool {
    if webview2_runtime_available() {
        log_line("WebView2 Runtime: detected");
        return true;
    }

    log_line("WebView2 Runtime: NOT detected");

    let log_hint = log_path()
        .map(|p| format!("\n\nLog file:\n{}", p.display()))
        .unwrap_or_default();

    if try_launch_webview2_bootstrapper() {
        show_error(
            "QuickNotes — WebView2 required",
            &format!(
                "Microsoft Edge WebView2 Runtime was not detected.\n\n\
                 The WebView2 installer has been started.\n\
                 Finish the install, then run QuickNotes.exe again.\n\n\
                 Manual download:\n{WEBVIEW2_DOWNLOAD}{log_hint}"
            ),
        );
        return false;
    }

    show_error(
        "QuickNotes — WebView2 required",
        &format!(
            "Microsoft Edge WebView2 Runtime was not detected.\n\n\
             QuickNotes needs WebView2 for its UI.\n\n\
             Install \"Evergreen Runtime\" from:\n{WEBVIEW2_DOWNLOAD}\n\n\
             Corporate PCs: ask IT to allow WebView2 / Edge Runtime.\n\
             Or run MicrosoftEdgeWebview2Setup.exe next to QuickNotes.exe.\n\n\
             Click OK to try starting anyway (if Runtime is installed but undetected).{log_hint}"
        ),
    );
    true
}

#[cfg(windows)]
fn webview2_runtime_available_windows() -> bool {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
        KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY, REG_SZ,
    };

    let subkeys = [
        format!(r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{WEBVIEW2_CLIENT_GUID}"),
        format!(r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{WEBVIEW2_CLIENT_GUID}"),
    ];

    let roots = [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER];
    let views = [KEY_WOW64_64KEY, KEY_WOW64_32KEY, 0];

    for root in roots {
        for sub in &subkeys {
            let wide: Vec<u16> = std::ffi::OsStr::new(sub)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            for view in views {
                unsafe {
                    let mut hkey = std::ptr::null_mut();
                    let status =
                        RegOpenKeyExW(root, wide.as_ptr(), 0, KEY_READ | view, &mut hkey);
                    if status != ERROR_SUCCESS || hkey.is_null() {
                        continue;
                    }

                    let name: Vec<u16> = std::ffi::OsStr::new("pv")
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();
                    let mut ty = 0u32;
                    let mut data = [0u16; 64];
                    let mut data_bytes = (data.len() * 2) as u32;
                    let q = RegQueryValueExW(
                        hkey,
                        name.as_ptr(),
                        std::ptr::null_mut(),
                        &mut ty,
                        data.as_mut_ptr() as *mut u8,
                        &mut data_bytes,
                    );
                    RegCloseKey(hkey);

                    if q == ERROR_SUCCESS && (ty == REG_SZ || ty == 0) {
                        let nul = data.iter().position(|&c| c == 0).unwrap_or(data.len());
                        let ver = String::from_utf16_lossy(&data[..nul]);
                        let ver = ver.trim();
                        if !ver.is_empty() && ver != "0.0.0.0" {
                            log_line(&format!("WebView2 pv={ver} (registry)"));
                            return true;
                        }
                    }
                }
            }
        }
    }

    // Fallback: Edge WebView2 executable on common install paths.
    let local = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    let candidates = [
        local
            .join(r"Microsoft\EdgeWebView\Application")
            .join("msedgewebview2.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Microsoft\EdgeWebView\Application\msedgewebview2.exe"),
        PathBuf::from(r"C:\Program Files\Microsoft\EdgeWebView\Application\msedgewebview2.exe"),
    ];
    for path in candidates {
        if path.is_file() {
            log_line(&format!("WebView2 exe found: {}", path.display()));
            return true;
        }
        // Versioned folders under Application\
        if let Some(parent) = path.parent() {
            if parent.is_dir() {
                if let Ok(rd) = fs::read_dir(parent) {
                    for entry in rd.flatten() {
                        let p = entry.path().join("msedgewebview2.exe");
                        if p.is_file() {
                            log_line(&format!("WebView2 exe found: {}", p.display()));
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

#[cfg(windows)]
fn message_box(title: &str, message: &str) {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

    let title: Vec<u16> = std::ffi::OsStr::new(title)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let message: Vec<u16> = std::ffi::OsStr::new(message)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}
