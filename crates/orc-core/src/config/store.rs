use std::path::PathBuf;

use anyhow::{Context, Result};

use super::AppConfig;

/// ~/.orc/
pub fn orc_dir() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .context("failed to find home directory")?
        .join(".orc");
    Ok(dir)
}

/// ~/.orc/config.toml
pub fn config_path() -> Result<PathBuf> {
    Ok(orc_dir()?.join("config.toml"))
}

/// ~/.orc/tokens/
pub fn tokens_dir() -> Result<PathBuf> {
    let dir = orc_dir()?.join("tokens");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
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
    let content = toml::to_string_pretty(config).context("failed to serialize config")?;
    std::fs::write(&path, content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}
