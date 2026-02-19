mod app_state;
mod audio;
mod command_parser;
mod commands;
mod config;
mod injector;
mod model;
mod platform;
mod transcription;
mod types;

use app_state::AppState;
use tauri::Manager;
use tauri_plugin_global_shortcut::ShortcutState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            #[cfg(desktop)]
            {
                let state = app.state::<AppState>();
                let hotkey = state.with_lock(|s| s.config.hotkey.clone());
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcuts([hotkey.as_str()])?
                        .with_handler(|app, _shortcut, event| {
                            if event.state != ShortcutState::Pressed {
                                return;
                            }
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let state = app_handle.state::<AppState>();
                                let result = commands::toggle_recording_internal(&state).await;
                                if let Err(err) = result {
                                    state.with_lock(|s| {
                                        s.status.last_error = Some(err);
                                    });
                                }
                            });
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
