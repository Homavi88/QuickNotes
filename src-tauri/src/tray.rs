use crate::startup;
use crate::windows::{show_main, show_quick};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use tauri_plugin_opener::OpenerExt;

pub fn setup_tray(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let quick = MenuItem::with_id(app, "quick", "Быстрая заметка", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let open_notes = MenuItem::with_id(app, "open_notes", "Открыть заметки", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings = MenuItem::with_id(app, "settings", "Настройки", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let open_folder =
        MenuItem::with_id(app, "open_folder", "Открыть папку с заметками", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let open_log = MenuItem::with_id(app, "open_log", "Открыть лог запуска", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        app,
        &[
            &quick,
            &open_notes,
            &settings,
            &open_folder,
            &open_log,
            &quit,
        ],
    )
    .map_err(|e| e.to_string())?;

    let tooltip = format!("QuickNotes ({hotkey})");
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "Missing default window icon".to_string())?;

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .tooltip(&tooltip)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quick" => {
                if let Err(e) = show_quick(app) {
                    startup::log_line(&format!("tray quick failed: {e}"));
                }
            }
            "open_notes" => {
                if let Err(e) = show_main(app, Some("notes")) {
                    startup::log_line(&format!("tray open_notes failed: {e}"));
                }
            }
            "settings" => {
                if let Err(e) = show_main(app, Some("settings")) {
                    startup::log_line(&format!("tray settings failed: {e}"));
                }
            }
            "open_folder" => {
                if let Some(state) = app.try_state::<crate::state::AppState>() {
                    let dir = state.config.lock().settings.storage_dir.clone();
                    let _ = app.opener().open_path(dir, None::<&str>);
                }
            }
            "open_log" => {
                if let Some(path) = startup::log_path() {
                    let _ = app.opener().open_path(path.to_string_lossy().to_string(), None::<&str>);
                }
            }
            "quit" => {
                startup::log_line("tray: quit");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Err(e) = show_quick(tray.app_handle()) {
                    startup::log_line(&format!("tray left-click failed: {e}"));
                }
            }
        })
        .build(app)
        .map_err(|e| format!("Failed to create tray: {e}"))?;

    Ok(())
}

pub fn update_tray_tooltip(app: &AppHandle, hotkey: &str) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(format!("QuickNotes ({hotkey})")));
    }
}
