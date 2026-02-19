use anyhow::{Context, Result};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::model::ensure_model_async;

#[derive(Debug, Clone)]
pub struct WhisperOutput {
  pub text: String,
  pub confidence: f32,
}

pub async fn transcribe_locally(pcm: Vec<f32>, model: &str) -> Result<WhisperOutput> {
  if pcm.is_empty() {
    return Ok(WhisperOutput {
      text: String::new(),
      confidence: 0.0,
    });
  }

  let model_path = ensure_model_async(model).await?;
  let model_path = model_path
    .to_str()
    .context("invalid model path for whisper runtime")?;

  let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
    .context("failed to initialize whisper context")?;
  let mut state = ctx
    .create_state()
    .context("failed to create whisper state")?;

  let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
  params.set_language(Some("en"));
  params.set_translate(false);
  params.set_print_special(false);
  params.set_print_progress(false);
  params.set_print_realtime(false);
  params.set_print_timestamps(false);
  params.set_n_threads(4);

  state
    .full(params, &pcm)
    .context("whisper inference failed")?;

  let num_segments = state
    .full_n_segments()
    .context("failed to read whisper segments")?;
  let mut text = String::new();
  for i in 0..num_segments {
    let segment = state
      .full_get_segment_text(i)
      .context("failed to read whisper segment text")?;
    text.push_str(segment.trim());
    text.push(' ');
  }

  Ok(WhisperOutput {
    text: text.trim().to_string(),
    confidence: 0.9,
  })
}
