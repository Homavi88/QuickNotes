mod commands;
mod config;
mod hotkey;
mod models;
mod notes;
mod state;
mod tray;
mod windows;

use config::load_config;
use models::default_storage_dir;
use state::AppState;
use std::fs;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::ShortcutState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                let _ = windows::show_quick(app);
            }))
            .plugin(
                tauri_plugin_window_state::Builder::new()
                    .with_filter(|label| label == "main")
                    .build(),
            )
            // Registers HKCU Run with the *current* exe path — works for portable
            // folders and MSI installs alike (no installer required for autostart).
            .plugin(
                tauri_plugin_autostart::Builder::new()
                    .app_name("QuickNotes")
                    .build(),
            )
            .plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(|app, _shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            let _ = windows::show_quick(app);
                        }
                    })
                    .build(),
            );
    }

    builder
        .manage(AppState::new(models::AppConfig::default()))
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::update_settings,
            commands::list_projects,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::reorder_projects,
            commands::list_notes,
            commands::save_note,
            commands::update_note,
            commands::delete_note,
            commands::save_draft,
            commands::set_last_project,
            commands::get_project_folder_path,
            commands::get_storage_dir,
            commands::get_runtime_paths,
            commands::show_quick_window,
            commands::hide_quick_window,
            commands::show_main_window,
            commands::apply_glass_effect,
            commands::reregister_hotkey,
        ])
        .setup(|app| {
            let mut cfg = load_config().unwrap_or_else(|e| {
                eprintln!("config load failed: {e}");
                models::AppConfig::default()
            });

            if cfg.settings.storage_dir.trim().is_empty() {
                cfg.settings.storage_dir = default_storage_dir();
            }
            if let Err(e) = fs::create_dir_all(&cfg.settings.storage_dir) {
                eprintln!("storage dir create failed: {e}");
            }
            if let Err(e) = config::save_config(&cfg) {
                eprintln!("config save failed: {e}");
            }

            {
                let state = app.state::<AppState>();
                *state.config.lock() = cfg.clone();
            }

            windows::create_windows(app.handle())?;

            if let Some(main) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                main.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(win) = app_handle.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                });
            }

            if let Some(quick) = app.get_webview_window("quick") {
                let app_handle = app.handle().clone();
                quick.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        let _ = app_handle.emit("quick-blur", ());
                    }
                });
            }

            tray::setup_tray(app.handle(), &cfg.settings.hotkey)?;

            #[cfg(desktop)]
            {
                hotkey::try_register_hotkey(app.handle(), &cfg.settings.hotkey);
            }

            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::ManagerExt;
                let autolaunch = app.autolaunch();
                if cfg.settings.autostart {
                    let _ = autolaunch.enable();
                }
            }

            if cfg.settings.glass_effect {
                let _ = windows::apply_glass(app.handle(), true);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running QuickNotes");
}
