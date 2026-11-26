use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn create_windows(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window("main").is_none() {
        WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
            .title("QuickNotes")
            .inner_size(1000.0, 680.0)
            .min_inner_size(720.0, 480.0)
            .resizable(true)
            // Tray-first: stay hidden until the user opens Notes from the tray / hotkey.
            .visible(false)
            .skip_taskbar(true)
            .center()
            .build()
            .map_err(|e| format!("Failed to create main window: {e}"))?;
    }

    // Quick capture — created hidden, reused via show/hide.
    if app.get_webview_window("quick").is_none() {
        let builder =
            WebviewWindowBuilder::new(app, "quick", WebviewUrl::App("index.html".into()))
                .title("QuickNotes Capture")
                .inner_size(520.0, 260.0)
                .resizable(false)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .visible(false)
                .focused(false)
                .center();

        #[cfg(target_os = "windows")]
        let builder = {
            use tauri::window::{Effect, EffectsBuilder};
            builder.effects(
                EffectsBuilder::new()
                    .effects([Effect::Acrylic, Effect::Mica, Effect::Blur])
                    .build(),
            )
        };

        builder
            .build()
            .map_err(|e| format!("Failed to create quick window: {e}"))?;
    }

    Ok(())
}

pub fn show_quick(app: &AppHandle) -> Result<(), String> {
    let Some(win) = app.get_webview_window("quick") else {
        return Err("Quick window missing".into());
    };
    let _ = win.center();
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    let _ = app.emit("quick-shown", ());
    Ok(())
}

pub fn hide_quick(app: &AppHandle) -> Result<(), String> {
    let Some(win) = app.get_webview_window("quick") else {
        return Ok(());
    };
    win.hide().map_err(|e| e.to_string())?;
    let _ = app.emit("quick-hidden", ());
    Ok(())
}

pub fn show_main(app: &AppHandle, section: Option<&str>) -> Result<(), String> {
    let Some(win) = app.get_webview_window("main") else {
        return Err("Main window missing".into());
    };
    let _ = win.unminimize();
    let _ = win.set_skip_taskbar(false);
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    if let Some(section) = section {
        let _ = app.emit("navigate-section", section.to_string());
    }
    Ok(())
}

pub fn apply_glass(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let Some(win) = app.get_webview_window("quick") else {
        return Ok(());
    };

    #[cfg(target_os = "windows")]
    {
        use tauri::window::{Effect, EffectsBuilder};
        if enabled {
            win.set_effects(Some(
                EffectsBuilder::new()
                    .effects([Effect::Acrylic, Effect::Mica, Effect::Blur])
                    .build(),
            ))
            .map_err(|e| e.to_string())?;
        } else {
            win.set_effects(None).map_err(|e| e.to_string())?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (win, enabled);
    }

    Ok(())
}
