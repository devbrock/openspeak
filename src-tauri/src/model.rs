use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use tokio::io::AsyncWriteExt;

const APP_DATA_DIR: &str = "openspeak";
const LEGACY_APP_DATA_DIR: &str = "brocks-dictation-tool";

fn model_root() -> Result<PathBuf> {
    let mut dir = dirs::data_local_dir().context("failed to locate local data directory")?;
    let mut legacy = dir.clone();
    legacy.push(LEGACY_APP_DATA_DIR);
    legacy.push("models");

    dir.push(APP_DATA_DIR);
    dir.push("models");
    fs::create_dir_all(&dir).context("failed to create model directory")?;

    if !dir.read_dir().map(|mut it| it.next().is_some()).unwrap_or(false) && legacy.exists() {
        return Ok(legacy);
    }

    Ok(dir)
}

fn model_filename(model_id: &str) -> Option<&'static str> {
    match model_id {
        "tiny" => Some("ggml-tiny.en.bin"),
        "base" => Some("ggml-base.en.bin"),
        "large" => Some("ggml-large-v3.bin"),
        _ => None,
    }
}

fn model_url(model_id: &str) -> Option<&'static str> {
    match model_id {
    "tiny" => Some(
      "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin?download=true",
    ),
    "base" => Some(
      "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin?download=true",
    ),
    "large" => Some(
      "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin?download=true",
    ),
    _ => None,
  }
}

pub fn model_path(model_id: &str) -> Result<PathBuf> {
    let file = model_filename(model_id).context("unknown model id")?;
    let mut path = model_root()?;
    path.push(file);
    Ok(path)
}

pub fn is_model_installed(model_id: &str) -> bool {
    let Ok(path) = model_path(model_id) else {
        return false;
    };
    path.exists()
}

pub async fn ensure_model_async(model_id: &str) -> Result<PathBuf> {
    if is_model_installed(model_id) {
        return model_path(model_id);
    }
    let _ = download_model(model_id).await?;
    model_path(model_id)
}

pub async fn download_model(model_id: &str) -> Result<String> {
    let path = model_path(model_id)?;
    let url = model_url(model_id).context("unknown model id")?;

    if path.exists() {
        return Ok(format!("existing-{}", model_id));
    }

    let tmp_path = path.with_extension("part");
    let client = reqwest::Client::builder()
        .build()
        .context("failed to build HTTP client")?;
    let bytes = client
        .get(url)
        .send()
        .await
        .context("failed to start model download")?
        .error_for_status()
        .context("model download failed with non-success status")?
        .bytes()
        .await
        .context("failed to read model download body")?;

    let mut file = tokio::fs::File::create(&tmp_path)
        .await
        .context("failed to create temporary model file")?;
    file.write_all(&bytes)
        .await
        .context("failed while writing model file")?;
    file.flush().await.context("failed flushing model file")?;

    fs::rename(&tmp_path, &path).context("failed finalizing downloaded model file")?;
    Ok(format!("downloaded-{}", model_id))
}
