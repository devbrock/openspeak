use tauri::{
    App, AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};

const OVERLAY_LABEL: &str = "recording-overlay";
const OVERLAY_WIDTH: f64 = 420.0;
const OVERLAY_HEIGHT: f64 = 132.0;
const BOTTOM_MARGIN: f64 = 28.0;

pub fn setup_overlay_window(app: &App) -> Result<(), String> {
    #[cfg(desktop)]
    {
        if app.get_webview_window(OVERLAY_LABEL).is_some() {
            return Ok(());
        }

        let monitor = app
            .primary_monitor()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "no monitor available for overlay".to_string())?;
        let monitor_size = monitor.size();
        let x = ((monitor_size.width as f64 - OVERLAY_WIDTH) / 2.0).max(0.0);
        let y = (monitor_size.height as f64 - OVERLAY_HEIGHT - BOTTOM_MARGIN).max(0.0);

        let window = WebviewWindowBuilder::new(
            app,
            OVERLAY_LABEL,
            WebviewUrl::App("index.html?overlay=1".into()),
        )
        .title("Dictation Overlay")
        .inner_size(OVERLAY_WIDTH, OVERLAY_HEIGHT)
        .position(x, y)
        .visible(false)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .closable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .skip_taskbar(true)
        .focused(false)
        .build()
        .map_err(|e| e.to_string())?;

        let _ = window.set_ignore_cursor_events(true);
        let _ = window.set_size(PhysicalSize::new(OVERLAY_WIDTH, OVERLAY_HEIGHT));
        let _ = window.set_position(PhysicalPosition::new(x, y));
        let _ = window.on_window_event(|event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
            }
        });
    }

    Ok(())
}

pub fn set_overlay_visible(app: &AppHandle, visible: bool) {
    #[cfg(desktop)]
    if let Some(window) = app.get_webview_window(OVERLAY_LABEL) {
        if visible {
            let _ = window.show();
            let _ = window.unminimize();
        } else {
            let _ = window.hide();
        }
    }
}
