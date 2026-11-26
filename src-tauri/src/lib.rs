mod commands;
mod config;
mod data_root;
mod hotkey;
mod models;
mod notes;
mod state;
mod startup;
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
    // File log first — release builds have no console on Windows.
    startup::init_logging();

    // Portable Fixed Runtime: set WEBVIEW2_BROWSER_EXECUTABLE_FOLDER before any webview.
    let _ = startup::prepare_fixed_webview2();

    // Missing WebView2 = invisible exit unless we stop here with a MessageBox.
    if !startup::ensure_webview2_or_explain() {
        startup::log_line("Exiting: WebView2 Runtime missing");
        return;
    }

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                startup::log_line("single-instance: focusing existing app");
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

    let result = builder
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
            commands::relocate_data_root,
            commands::reset_data_root,
            commands::show_quick_window,
            commands::hide_quick_window,
            commands::show_main_window,
            commands::apply_glass_effect,
            commands::reregister_hotkey,
        ])
        .setup(|app| {
            startup::log_line("setup: begin");

            let mut cfg = load_config().unwrap_or_else(|e| {
                startup::log_line(&format!("config load failed: {e}"));
                models::AppConfig::default()
            });

            if cfg.settings.storage_dir.trim().is_empty() {
                cfg.settings.storage_dir = default_storage_dir();
            }
            if let Err(e) = fs::create_dir_all(&cfg.settings.storage_dir) {
                startup::log_line(&format!("storage dir create failed: {e}"));
                startup::show_error(
                    "QuickNotes — storage folder",
                    &format!(
                        "Could not create the notes folder:\n{}\n\n{e}\n\n\
                         Unzip to a writable location (e.g. your user profile), not a read-only network share.",
                        cfg.settings.storage_dir
                    ),
                );
            }
            if let Err(e) = config::save_config(&cfg) {
                startup::log_line(&format!("config save failed: {e}"));
                startup::show_error(
                    "QuickNotes — config",
                    &format!(
                        "Could not write config.json.\n\n{e}\n\n\
                         Ensure the folder is writable (portable mode uses <exe>\\data\\)."
                    ),
                );
            }

            {
                let state = app.state::<AppState>();
                *state.config.lock() = cfg.clone();
            }

            // Tray BEFORE webviews — if WebView creation fails, user still sees an icon.
            startup::log_line("setup: creating tray");
            if let Err(e) = tray::setup_tray(app.handle(), &cfg.settings.hotkey) {
                startup::show_error(
                    "QuickNotes — tray",
                    &format!(
                        "Could not create the system tray icon.\n\n{e}\n\n\
                         Check that the notification area is not hidden, then relaunch."
                    ),
                );
                // Still continue — windows may work even if tray failed.
            } else {
                startup::log_line("setup: tray ready");
            }

            startup::log_line("setup: creating windows");
            match windows::create_windows(app.handle()) {
                Ok(()) => startup::log_line("setup: windows ready"),
                Err(e) => {
                    startup::show_error(
                        "QuickNotes — window error",
                        &format!(
                            "Failed to create app windows (WebView2 may be missing or blocked).\n\n{e}\n\n\
                             Portable: confirm WebView2\\ sits next to QuickNotes.exe.\n\
                             Do not run from a UNC/network path. Prefer a local disk folder.\n\
                             See data\\quicknotes.log next to the exe for details."
                        ),
                    );
                    // Keep running so the tray Quit item still works.
                }
            }

            if let Some(main) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                main.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(win) = app_handle.get_webview_window("main") {
                            let _ = win.hide();
                            let _ = win.set_skip_taskbar(true);
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

            #[cfg(desktop)]
            {
                hotkey::try_register_hotkey(app.handle(), &cfg.settings.hotkey);
            }

            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::ManagerExt;
                let autolaunch = app.autolaunch();
                if cfg.settings.autostart {
                    if let Err(e) = autolaunch.enable() {
                        startup::log_line(&format!("autostart enable failed: {e}"));
                    }
                }
            }

            if cfg.settings.glass_effect {
                let _ = windows::apply_glass(app.handle(), true);
            }

            startup::log_line("setup: complete");
            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(e) = result {
        startup::show_error(
            "QuickNotes — failed to start",
            &format!(
                "The application could not start.\n\n{e}\n\n\
                 Portable builds need WebView2\\ next to QuickNotes.exe \
                 (no separate Runtime installer).\n\n\
                 Check quicknotes.log beside the exe (or under data\\)."
            ),
        );
    }
}
