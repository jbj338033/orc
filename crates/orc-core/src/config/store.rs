use std::path::PathBuf;

use anyhow::{Context, Result};

use super::AppConfig;

fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("failed to find config directory")?
        .join("orc");
    Ok(dir)
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let config: AppConfig =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content =
        toml::to_string_pretty(config).context("failed to serialize config")?;
    std::fs::write(&path, content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}
