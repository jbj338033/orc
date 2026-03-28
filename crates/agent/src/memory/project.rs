use std::collections::HashMap;
use std::path::Path;

use super::types::ProjectMemoryEntry;

pub struct ProjectMemory {
    entries: HashMap<String, ProjectMemoryEntry>,
    path: std::path::PathBuf,
}

impl ProjectMemory {
    pub fn new(path: std::path::PathBuf) -> Self {
        let entries = Self::load_from_disk(&path).unwrap_or_default();
        Self { entries, path }
    }

    pub fn get(&self, key: &str) -> Option<&ProjectMemoryEntry> {
        self.entries.get(key)
    }

    pub fn set(&mut self, key: String, value: String) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.entries.insert(
            key.clone(),
            ProjectMemoryEntry {
                key,
                value,
                updated_at: now,
            },
        );
        let _ = self.save_to_disk();
    }

    pub fn remove(&mut self, key: &str) {
        self.entries.remove(key);
        let _ = self.save_to_disk();
    }

    pub fn all(&self) -> Vec<&ProjectMemoryEntry> {
        self.entries.values().collect()
    }

    pub fn to_context_string(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }
        let mut lines = vec!["[Project Memory]".to_string()];
        for entry in self.entries.values() {
            lines.push(format!("- {}: {}", entry.key, entry.value));
        }
        lines.join("\n")
    }

    fn load_from_disk(path: &Path) -> Option<HashMap<String, ProjectMemoryEntry>> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save_to_disk(&self) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.entries)?;
        std::fs::write(&self.path, content)
    }
}
