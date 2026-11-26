use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub fn register_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let _ = app.global_shortcut().unregister_all();
    let shortcut: Shortcut = hotkey
        .parse()
        .map_err(|e| format!("Некорректный хоткей '{hotkey}': {e}"))?;
    app.global_shortcut()
        .register(shortcut)
        .map_err(|e| format!("Не удалось зарегистрировать хоткей: {e}"))?;
    crate::tray::update_tray_tooltip(app, hotkey);
    let _ = app.emit("hotkey-registered", hotkey.to_string());
    Ok(())
}

/// Best-effort parse helper used at startup; errors are logged, not fatal.
pub fn try_register_hotkey(app: &AppHandle, hotkey: &str) {
    if let Err(e) = register_hotkey(app, hotkey) {
        eprintln!("{e}");
        let _ = app.emit("hotkey-error", e);
    }
}
