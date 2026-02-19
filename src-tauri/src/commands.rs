use std::time::Instant;

use tauri::State;

use crate::{
  app_state::AppState,
  audio::{finalize_capture, RecordingSession},
  command_parser::apply_basic_commands,
  config::save_config,
  injector::copy_text_to_clipboard,
  model::{download_model as download_model_file, is_model_installed},
  transcription::transcribe_locally,
  types::{AppConfig, AppStatus, RecordingState, TranscriptionResult},
};

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> Result<AppStatus, String> {
  let status = state.with_lock(|s| {
    let mut out = s.status.clone();
    out.model_ready = is_model_installed(&s.config.model_default);
    out
  });
  Ok(status)
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
  Ok(state.with_lock(|s| s.config.clone()))
}

#[tauri::command]
pub fn set_hotkey(state: State<'_, AppState>, hotkey_spec: String) -> Result<(), String> {
  state.with_lock(|s| {
    s.config.hotkey = hotkey_spec;
    save_config(&s.config).map_err(|e| e.to_string())
  })
}

#[tauri::command]
pub fn set_model(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
  if !matches!(model_id.as_str(), "tiny" | "base" | "large") {
    return Err("invalid model id".to_string());
  }
  state.with_lock(|s| {
    s.config.model_default = model_id;
    save_config(&s.config).map_err(|e| e.to_string())
  })
}

#[tauri::command]
pub async fn download_model(model_id: String) -> Result<String, String> {
  download_model_file(&model_id)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_recording(state: State<'_, AppState>) -> Result<String, String> {
  state.with_lock(|s| {
    if s.active_session.is_some() {
      return Err("recording session already active".to_string());
    }
    let session = RecordingSession::begin().map_err(|e| e.to_string())?;
    let id = session.id.to_string();
    s.active_session = Some(session);
    s.status.recording_state = RecordingState::Recording;
    Ok(id)
  })
}

#[tauri::command]
pub async fn stop_recording(
  state: State<'_, AppState>,
  session_id: String,
) -> Result<TranscriptionResult, String> {
  let maybe_session = state.with_lock(|s| {
    s.status.recording_state = RecordingState::Transcribing;
    s.active_session.take()
  });
  let session = maybe_session.ok_or_else(|| "no active recording session".to_string())?;
  if session.id.to_string() != session_id {
    return Err("session id mismatch".to_string());
  }

  let start = Instant::now();
  let session_elapsed_ms = session.elapsed_ms();
  let pcm = finalize_capture(session).await.map_err(|e| e.to_string())?;
  let model_default = state.with_lock(|s| s.config.model_default.clone());
  let whisper = transcribe_locally(pcm, &model_default)
    .await
    .map_err(|e| e.to_string())?;
  let parsed = apply_basic_commands(&whisper.text);
  copy_text_to_clipboard(&parsed.transformed_text).map_err(|e| e.to_string())?;

  let result = TranscriptionResult {
    raw_text: whisper.text,
    transformed_text: parsed.transformed_text,
    commands_applied: parsed.commands_applied,
    latency_ms: start.elapsed().as_millis() + session_elapsed_ms,
    confidence: whisper.confidence,
  };

  state.with_lock(|s| {
    s.status.recording_state = RecordingState::Idle;
  });

  Ok(result)
}
