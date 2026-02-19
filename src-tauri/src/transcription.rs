use anyhow::{Context, Result};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::model::{download_model, ensure_model_async, model_path};

#[derive(Debug, Clone)]
pub struct WhisperOutput {
    pub text: String,
    pub confidence: f32,
}

fn clean_whisper_text(input: &str) -> String {
    let mut out = Vec::new();
    for token in input.split_whitespace() {
        if token.eq_ignore_ascii_case("[BLANK_AUDIO]") {
            continue;
        }
        out.push(token);
    }
    out.join(" ").trim().to_string()
}

fn run_inference(pcm: &[f32], model_path: &str) -> Result<WhisperOutput> {
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
        .full(params, pcm)
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
        text: clean_whisper_text(&text),
        confidence: 0.9,
    })
}

pub async fn transcribe_locally(pcm: Vec<f32>, model: &str) -> Result<WhisperOutput> {
    if pcm.is_empty() {
        return Ok(WhisperOutput {
            text: String::new(),
            confidence: 0.0,
        });
    }

    let initial_model_path = ensure_model_async(model).await?;
    let initial_model_path = initial_model_path
        .to_str()
        .context("invalid model path for whisper runtime")?;

    match run_inference(&pcm, initial_model_path) {
        Ok(output) => Ok(output),
        Err(first_error) => {
            // If the model is corrupted/incomplete, refresh it once and retry.
            if let Ok(path) = model_path(model) {
                let _ = std::fs::remove_file(path);
            }
            download_model(model).await?;
            let refreshed = ensure_model_async(model).await?;
            let refreshed = refreshed
                .to_str()
                .context("invalid refreshed model path for whisper runtime")?;
            run_inference(&pcm, refreshed)
                .with_context(|| format!("transcription failed after model refresh: {first_error}"))
        }
    }
}
