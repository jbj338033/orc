use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    #[serde(default)]
    pub provider: Vec<ProviderEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_provider: None,
            default_model: None,
            provider: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub base_url: Option<String>,
    #[serde(default)]
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    pub method: Option<String>,
    pub api_key: Option<String>,
    pub api_key_env: Option<String>,
}

impl AuthConfig {
    pub fn resolve_api_key(&self) -> Option<String> {
        if let Some(key) = &self.api_key {
            return Some(key.clone());
        }
        if let Some(env_name) = &self.api_key_env {
            return std::env::var(env_name).ok();
        }
        None
    }
}
