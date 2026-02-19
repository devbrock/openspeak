use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

use crate::types::{AppConfig, PrivacyConfig};

const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedConfig {
    config_version: u32,
    #[serde(flatten)]
    config: AppConfig,
}

pub fn default_config() -> AppConfig {
    AppConfig {
    hotkey: "CommandOrControl+Shift+Space".to_string(),
        model_default: "tiny".to_string(),
        command_mode: "basic".to_string(),
        paste_mode: "clipboard".to_string(),
        language: "en".to_string(),
        privacy: PrivacyConfig {
            telemetry_enabled: false,
            persist_audio_debug: false,
        },
    }
}

fn config_path() -> Result<PathBuf> {
    let mut dir = dirs::data_local_dir().context("failed to locate local data directory")?;
    dir.push("brocks-dictation-tool");
    fs::create_dir_all(&dir).context("failed to create app data directory")?;
    dir.push("config.json");
    Ok(dir)
}

pub fn load_or_init_config() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        let config = default_config();
        save_config(&config)?;
        return Ok(config);
    }

    let raw = fs::read_to_string(&path).context("failed to read config file")?;
    let persisted: PersistedConfig =
        serde_json::from_str(&raw).context("failed to parse config file JSON")?;

    if persisted.config_version != CONFIG_VERSION {
        let cfg = default_config();
        save_config(&cfg)?;
        return Ok(cfg);
    }

    Ok(persisted.config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path()?;
    let persisted = PersistedConfig {
        config_version: CONFIG_VERSION,
        config: config.clone(),
    };
    let content = serde_json::to_string_pretty(&persisted).context("failed to serialize config")?;
    fs::write(path, content).context("failed to write config")
}
