//! Early startup helpers: file logging, WebView2 Fixed Runtime, native error dialogs.
//!
//! Portable / corporate machines often have no console (`windows_subsystem = "windows"`),
//! so silent exits are useless. Always write `<exe>/data/quicknotes.log` (portable) or
//! beside the exe / AppData, and show a MessageBox for fatal runtime gaps.
//!
//! Portable packages ship a Fixed Version WebView2 Runtime under `<exe>/WebView2/`.
//! We point WebView2 at it via `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` — no MSI / setup exe.

use crate::models::{config_dir, exe_dir, is_portable_mode};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
#[cfg(windows)]
use std::path::Path;
use std::sync::Mutex;

static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

const WEBVIEW2_DOWNLOAD: &str = "https://developer.microsoft.com/microsoft-edge/webview2/";
const WEBVIEW2_CLIENT_GUID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
/// Stable folder name written by `scripts/package-portable.ps1`.
const FIXED_RUNTIME_DIR: &str = "WebView2";

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

/// Locate a bundled Fixed Version runtime folder next to the exe.
///
/// Looks for:
/// - `<exe>/WebView2/msedgewebview2.exe` (portable packaging)
/// - `<exe>/Microsoft.WebView2.FixedVersionRuntime.*/msedgewebview2.exe`
pub fn find_fixed_runtime_dir() -> Option<PathBuf> {
    let dir = exe_dir()?;

    let stable = dir.join(FIXED_RUNTIME_DIR);
    if stable.join("msedgewebview2.exe").is_file() {
        return Some(stable);
    }

    if let Ok(rd) = fs::read_dir(&dir) {
        for entry in rd.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("Microsoft.WebView2.FixedVersionRuntime") {
                let candidate = entry.path();
                if candidate.join("msedgewebview2.exe").is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

/// Point WebView2 at the bundled Fixed Runtime (and portable user-data folder) **before**
/// Tauri/wry creates any webview. Must run early in `run()`.
///
/// Returns `true` when a Fixed Runtime folder was found and env vars were set.
pub fn prepare_fixed_webview2() -> bool {
    let Some(runtime) = find_fixed_runtime_dir() else {
        log_line("Fixed WebView2 Runtime: not bundled next to exe");
        return false;
    };

    let runtime_str = runtime.to_string_lossy().into_owned();
    log_line(&format!(
        "Fixed WebView2 Runtime: {}",
        runtime.display()
    ));

    // Required so CreateCoreWebView2Environment uses the local binaries (no system install).
    std::env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", &runtime_str);

    // Keep WebView2 profile/cache under the portable data/ tree when possible.
    if is_portable_mode() {
        let user_data = config_dir().join("webview2");
        if let Err(e) = fs::create_dir_all(&user_data) {
            log_line(&format!("webview2 user-data dir create failed: {e}"));
        } else {
            std::env::set_var(
                "WEBVIEW2_USER_DATA_FOLDER",
                user_data.to_string_lossy().as_ref(),
            );
            log_line(&format!(
                "WEBVIEW2_USER_DATA_FOLDER={}",
                user_data.display()
            ));
        }
    }

    #[cfg(windows)]
    {
        ensure_fixed_runtime_acls(&runtime);
    }

    log_line(&format!(
        "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER={}",
        runtime_str
    ));
    true
}

/// Returns true when Fixed Runtime is bundled **or** Evergreen Runtime appears installed.
pub fn webview2_runtime_available() -> bool {
    if find_fixed_runtime_dir().is_some() {
        return true;
    }
    #[cfg(windows)]
    {
        webview2_runtime_available_windows()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// Pre-flight before Tauri window creation.
///
/// With a bundled Fixed Runtime this always succeeds (no installer needed).
/// Without it, falls back to system Evergreen detection + MessageBox guidance.
/// Always returns `true` so a false-negative registry check cannot brick startup —
/// window creation will MessageBox again if WebView2 truly fails.
pub fn ensure_webview2_or_explain() -> bool {
    if find_fixed_runtime_dir().is_some() {
        log_line("WebView2 Runtime: Fixed Version bundled (no installer required)");
        return true;
    }

    if webview2_runtime_available_system() {
        log_line("WebView2 Runtime: system Evergreen detected");
        return true;
    }

    log_line("WebView2 Runtime: NOT detected (no Fixed Runtime folder, no system Runtime)");

    let log_hint = log_path()
        .map(|p| format!("\n\nLog file:\n{}", p.display()))
        .unwrap_or_default();

    show_error(
        "QuickNotes — WebView2 required",
        &format!(
            "Microsoft Edge WebView2 Runtime was not detected.\n\n\
             This portable build should include a WebView2\\ folder next to QuickNotes.exe.\n\
             Re-download the portable zip from GitHub Releases (v0.1.3+) and unzip again.\n\n\
             Or install Evergreen Runtime from:\n{WEBVIEW2_DOWNLOAD}\n\n\
             Click OK to try starting anyway.{log_hint}"
        ),
    );
    true
}

fn webview2_runtime_available_system() -> bool {
    #[cfg(windows)]
    {
        webview2_runtime_available_windows()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// Windows 10 + Fixed Version ≥120 needs App Container ACLs on the runtime folder.
/// Best-effort; failures are logged and do not abort startup (Windows 11 usually fine).
#[cfg(windows)]
fn ensure_fixed_runtime_acls(runtime: &Path) {
    let marker = runtime.join(".quicknotes-acls-ok");
    if marker.is_file() {
        log_line("Fixed Runtime ACLs: already applied");
        return;
    }

    let path = runtime.to_string_lossy();
    // ALL APPLICATION PACKAGES + ALL RESTRICTED APPLICATION PACKAGES (RX, inherit)
    let grants = [
        ("S-1-15-2-2", "(OI)(CI)(RX)"),
        ("S-1-15-2-1", "(OI)(CI)(RX)"),
    ];

    for (sid, rights) in grants {
        let grant = format!("*{sid}:{rights}");
        match std::process::Command::new("icacls")
            .arg(path.as_ref())
            .arg("/grant")
            .arg(&grant)
            .output()
        {
            Ok(out) => {
                let code = out.status.code().unwrap_or(-1);
                let stderr = String::from_utf8_lossy(&out.stderr);
                log_line(&format!(
                    "icacls grant {grant}: exit={code} stderr={}",
                    stderr.trim()
                ));
            }
            Err(e) => log_line(&format!("icacls failed to start ({grant}): {e}")),
        }
    }

    let _ = fs::write(&marker, b"ok\n");
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
