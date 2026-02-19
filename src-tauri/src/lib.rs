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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(AppState::new())
    .invoke_handler(tauri::generate_handler![
      commands::get_status,
      commands::get_config,
      commands::start_recording,
      commands::stop_recording,
      commands::set_hotkey,
      commands::set_model,
      commands::download_model
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
