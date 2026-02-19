mod app_state;
mod audio;
mod command_parser;
mod commands;
mod config;
mod injector;
mod model;
mod overlay;
mod platform;
mod transcription;
mod types;

use app_state::AppState;
use tauri::{AppHandle, Manager, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

#[cfg(desktop)]
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
};
use crate::types::RecordingState;

const TRAY_TOGGLE_ID: &str = "tray_toggle_dictation";
const TRAY_OPEN_SETTINGS_ID: &str = "tray_open_settings";
const TRAY_QUIT_ID: &str = "tray_quit";
const TRAY_COLOR_PNG: &[u8] = include_bytes!("../icons/waveform.png");

fn show_settings_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    Err("settings window not available".to_string())
}

#[cfg(desktop)]
fn refresh_tray_toggle_label(state: &AppState, toggle_item: &MenuItem<tauri::Wry>) {
    let next_text = state.with_lock(|s| match s.status.recording_state {
        RecordingState::Idle => "Start Dictation",
        RecordingState::Recording | RecordingState::Transcribing => "Stop Dictation",
    });
    let _ = toggle_item.set_text(next_text);
}

#[cfg(desktop)]
fn trigger_toggle(app_handle: AppHandle, toggle_item: MenuItem<tauri::Wry>) {
    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<AppState>();
        let result = commands::toggle_recording_internal(&app_handle, &state).await;
        if let Err(err) = result {
            state.with_lock(|s| {
                s.status.last_error = Some(err);
            });
        }
        refresh_tray_toggle_label(&state, &toggle_item);
    });
}

#[cfg(desktop)]
fn setup_tray(app: &tauri::App) -> Result<MenuItem<tauri::Wry>, String> {
    let toggle =
        MenuItem::with_id(app, TRAY_TOGGLE_ID, "Start Dictation", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let open_settings =
        MenuItem::with_id(app, TRAY_OPEN_SETTINGS_ID, "Settings...", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let quit = MenuItem::with_id(app, TRAY_QUIT_ID, "Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;

    let menu = Menu::with_items(app, &[&toggle, &open_settings, &separator, &quit])
        .map_err(|e| e.to_string())?;

    let toggle_for_menu = toggle.clone();
    let mut tray = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("OpenSpeak")
        .icon_as_template(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            TRAY_TOGGLE_ID => {
                trigger_toggle(app.clone(), toggle_for_menu.clone());
            }
            TRAY_OPEN_SETTINGS_ID => {
                if let Err(err) = show_settings_window(app) {
                    let state = app.state::<AppState>();
                    state.with_lock(|s| {
                        s.status.last_error = Some(err);
                    });
                }
            }
            TRAY_QUIT_ID => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event({
            let toggle_for_events = toggle.clone();
            move |_tray, event| {
                if matches!(event, TrayIconEvent::Click { .. }) {
                    let state = toggle_for_events.app_handle().state::<AppState>();
                    refresh_tray_toggle_label(&state, &toggle_for_events);
                }
            }
        });

    if let Ok(icon) = Image::from_bytes(TRAY_COLOR_PNG) {
        tray = tray.icon(icon);
    } else if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }

    tray.build(app).map_err(|e| e.to_string())?;
    let state = app.state::<AppState>();
    refresh_tray_toggle_label(&state, &toggle);
    Ok(toggle)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            overlay::setup_overlay_window(app)?;
            #[cfg(desktop)]
            let tray_toggle_item = setup_tray(app)?;

            #[cfg(desktop)]
            {
                let state = app.state::<AppState>();
                let hotkey = state.with_lock(|s| s.config.hotkey.clone());
                let hotkey_toggle_item = tray_toggle_item.clone();
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcuts([hotkey.as_str()])?
                        .with_handler(move |app, _shortcut, event| {
                            if event.state != ShortcutState::Pressed {
                                return;
                            }
                            let app_handle = app.clone();
                            trigger_toggle(app_handle, hotkey_toggle_item.clone());
                        })
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_config,
            commands::start_recording,
            commands::stop_recording,
            commands::toggle_recording,
            commands::set_hotkey,
            commands::set_paste_mode,
            commands::set_model,
            commands::download_model
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
