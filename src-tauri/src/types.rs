use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordingState {
  Idle,
  Recording,
  Transcribing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
  pub recording_state: RecordingState,
  pub model_ready: bool,
  pub microphone_granted: bool,
  pub accessibility_granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyConfig {
  pub telemetry_enabled: bool,
  pub persist_audio_debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
  pub hotkey: String,
  pub model_default: String,
  pub command_mode: String,
  pub paste_mode: String,
  pub language: String,
  pub privacy: PrivacyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionResult {
  pub raw_text: String,
  pub transformed_text: String,
  pub commands_applied: Vec<String>,
  pub latency_ms: u128,
  pub confidence: f32,
}
