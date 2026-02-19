use std::time::Instant;

use tauri::{AppHandle, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::{
    app_state::AppState,
    audio::{finalize_capture, RecordingSession},
    command_parser::apply_basic_commands,
    config::save_config,
    injector::deliver_text,
    model::{download_model as download_model_file, is_model_installed, is_supported_model},
    overlay::set_overlay_visible,
    transcription::transcribe_locally,
    types::{AppConfig, AppStatus, RecordingState, TranscriptionResult},
};

fn set_last_error(state: &AppState, message: Option<String>) {
    state.with_lock(|s| {
        s.status.last_error = message;
    });
}

#[tauri::command]
pub fn get_status(state: State<'_, AppState>) -> Result<AppStatus, String> {
    let status = state.with_lock(|s| {
        #[cfg(target_os = "macos")]
        {
            s.status.accessibility_granted = crate::platform::macos::accessibility_granted();
        }
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
pub fn set_hotkey(
    app: AppHandle,
    state: State<'_, AppState>,
    hotkey_spec: String,
) -> Result<(), String> {
    let parsed_hotkey: Shortcut = hotkey_spec
        .parse()
        .map_err(|e| format!("invalid hotkey format: {e}"))?;

    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;
    app.global_shortcut()
        .register(parsed_hotkey)
        .map_err(|e| e.to_string())?;

    let result = state.with_lock(|s| {
        s.config.hotkey = hotkey_spec;
        save_config(&s.config).map_err(|e| e.to_string())
    });
    if let Err(err) = &result {
        set_last_error(&state, Some(err.clone()));
    } else {
        set_last_error(&state, None);
    }
    result
}

#[tauri::command]
pub fn reset_permissions(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let bundle_id = app.config().identifier.as_str();
        crate::platform::macos::reset_permissions(bundle_id).map_err(|e| e.to_string())?;
        state.with_lock(|s| {
            s.status.accessibility_granted = crate::platform::macos::accessibility_granted();
            s.status.last_error = None;
        });
    }
    Ok(())
}

#[tauri::command]
pub fn enable_permissions(state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        crate::platform::macos::open_permissions_settings().map_err(|e| e.to_string())?;
        let trusted_after_prompt =
            crate::platform::macos::prompt_accessibility_permission().map_err(|e| e.to_string())?;

        // Best-effort microphone prompt: initializing input capture causes macOS to ask
        // for Microphone permission if this app has not been granted yet.
        let mic_init_result = RecordingSession::begin();
        if let Err(err) = mic_init_result {
            state.with_lock(|s| {
                s.status.last_error = Some(format!(
                    "Microphone check failed while requesting permissions: {err}"
                ));
            });
        } else {
            state.with_lock(|s| {
                s.status.last_error = None;
                s.status.microphone_granted = true;
            });
        }

        state.with_lock(|s| {
            s.status.accessibility_granted = crate::platform::macos::accessibility_granted();
        });

        if !trusted_after_prompt && !crate::platform::macos::accessibility_granted() {
            return Err(
                "Accessibility permission is still not granted. In System Settings > Privacy & Security > Accessibility, enable OpenSpeak, then fully quit and relaunch the app."
                    .to_string(),
            );
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_model(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
    if !is_supported_model(&model_id) {
        return Err("invalid model id".to_string());
    }
    state.with_lock(|s| {
        s.config.model_default = model_id;
        save_config(&s.config).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn set_paste_mode(state: State<'_, AppState>, paste_mode: String) -> Result<(), String> {
    if !matches!(paste_mode.as_str(), "clipboard" | "auto-paste") {
        return Err("invalid paste mode".to_string());
    }
    let result = state.with_lock(|s| {
        s.config.paste_mode = paste_mode;
        save_config(&s.config).map_err(|e| e.to_string())
    });
    if let Err(err) = &result {
        set_last_error(&state, Some(err.clone()));
    } else {
        set_last_error(&state, None);
    }
    result
}

#[tauri::command]
pub async fn download_model(model_id: String) -> Result<String, String> {
    download_model_file(&model_id)
        .await
        .map_err(|e| e.to_string())
}

pub fn start_recording_internal(app: &AppHandle, state: &AppState) -> Result<String, String> {
    let result = state.with_lock(|s| {
        if s.active_session.is_some() {
            return Err("recording session already active".to_string());
        }
        let session = RecordingSession::begin().map_err(|e| e.to_string())?;
        let id = session.id.to_string();
        s.active_session = Some(session);
        s.status.recording_state = RecordingState::Recording;
        Ok(id)
    });
    if result.is_ok() {
        set_overlay_visible(app, true);
    }
    result
}

pub async fn stop_recording_internal(
    app: &AppHandle,
    state: &AppState,
    session_id: String,
) -> Result<TranscriptionResult, String> {
    let maybe_session = state.with_lock(|s| {
        let taken = s.active_session.take();
        if taken.is_some() {
            s.status.recording_state = RecordingState::Transcribing;
        }
        taken
    });
    let session = maybe_session.ok_or_else(|| "no active recording session".to_string())?;
    if session.id.to_string() != session_id {
        state.with_lock(|s| {
            s.status.recording_state = RecordingState::Idle;
        });
        set_overlay_visible(app, false);
        return Err("session id mismatch".to_string());
    }

    let result = async {
        let start = Instant::now();
        let session_elapsed_ms = session.elapsed_ms();
        let pcm = finalize_capture(session).await.map_err(|e| e.to_string())?;
        let (model_default, paste_mode) =
            state.with_lock(|s| (s.config.model_default.clone(), s.config.paste_mode.clone()));
        let whisper = transcribe_locally(pcm, &model_default)
            .await
            .map_err(|e| e.to_string())?;
        let parsed = apply_basic_commands(&whisper.text);
        let delivery =
            deliver_text(&parsed.transformed_text, &paste_mode).map_err(|e| e.to_string())?;

        Ok(TranscriptionResult {
            raw_text: whisper.text,
            transformed_text: parsed.transformed_text,
            commands_applied: parsed.commands_applied,
            latency_ms: start.elapsed().as_millis() + session_elapsed_ms,
            confidence: whisper.confidence,
            delivery,
        })
    }
    .await;

    state.with_lock(|s| {
        s.status.recording_state = RecordingState::Idle;
        s.status.last_error = result.as_ref().err().cloned();
    });
    set_overlay_visible(app, false);

    result
}

pub async fn toggle_recording_internal(
    app: &AppHandle,
    state: &AppState,
) -> Result<Option<TranscriptionResult>, String> {
    let maybe_session_id = state.with_lock(|s| {
        s.active_session
            .as_ref()
            .map(|session| session.id.to_string())
    });
    if let Some(id) = maybe_session_id {
        let result = stop_recording_internal(app, state, id).await?;
        return Ok(Some(result));
    }

    start_recording_internal(app, state)?;
    Ok(None)
}

#[tauri::command]
pub fn start_recording(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let result = start_recording_internal(&app, &state);
    if let Err(err) = &result {
        set_last_error(&state, Some(err.clone()));
    } else {
        set_last_error(&state, None);
    }
    result
}

#[tauri::command]
pub async fn stop_recording(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
) -> Result<TranscriptionResult, String> {
    let result = stop_recording_internal(&app, &state, session_id).await;
    if let Err(err) = &result {
        set_last_error(&state, Some(err.clone()));
    }
    result
}

#[tauri::command]
pub async fn toggle_recording(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<TranscriptionResult>, String> {
    let result = toggle_recording_internal(&app, &state).await;
    if let Err(err) = &result {
        set_last_error(&state, Some(err.clone()));
    } else {
        set_last_error(&state, None);
    }
    result
}
