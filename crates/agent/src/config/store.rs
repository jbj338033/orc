use std::fs;
use std::path::PathBuf;

use super::error::ConfigError;
use super::profile::AgentProfile;

pub trait ConfigStore: Send + Sync {
    fn load_profile(&self, id: &str) -> Result<AgentProfile, ConfigError>;
    fn save_profile(&self, profile: &AgentProfile) -> Result<(), ConfigError>;
    fn list_profiles(&self) -> Result<Vec<AgentProfile>, ConfigError>;
    fn delete_profile(&self, id: &str) -> Result<(), ConfigError>;
}

pub struct FileConfigStore {
    dir: PathBuf,
}

impl FileConfigStore {
    pub fn new(dir: PathBuf) -> Result<Self, ConfigError> {
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    fn profile_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }
}

impl ConfigStore for FileConfigStore {
    fn load_profile(&self, id: &str) -> Result<AgentProfile, ConfigError> {
        let path = self.profile_path(id);
        if !path.exists() {
            return Err(ConfigError::NotFound(id.to_string()));
        }
        let data = fs::read_to_string(&path)?;
        let profile: AgentProfile = serde_json::from_str(&data)?;
        Ok(profile)
    }

    fn save_profile(&self, profile: &AgentProfile) -> Result<(), ConfigError> {
        let path = self.profile_path(&profile.id);
        let data = serde_json::to_string_pretty(profile)?;
        fs::write(&path, data)?;
        Ok(())
    }

    fn list_profiles(&self) -> Result<Vec<AgentProfile>, ConfigError> {
        let mut profiles = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                let data = fs::read_to_string(&path)?;
                if let Ok(profile) = serde_json::from_str::<AgentProfile>(&data) {
                    profiles.push(profile);
                }
            }
        }
        Ok(profiles)
    }

    fn delete_profile(&self, id: &str) -> Result<(), ConfigError> {
        let path = self.profile_path(id);
        if !path.exists() {
            return Err(ConfigError::NotFound(id.to_string()));
        }
        fs::remove_file(&path)?;
        Ok(())
    }
}
