
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStore {
    pub entries: Vec<MemoryEntry>,
}

impl MemoryStore {
    pub fn new(path: PathBuf) -> Self {
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        let now = chrono::Utc::now().timestamp();
        self.entries.push(MemoryEntry { key: key.into(), value: value.into(), created_at: now });
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.iter().rev().find(|e| e.key == key).map(|e| e.value.as_str())
    }

    pub fn remove(&mut self, key: &str) {
        self.entries.retain(|e| e.key != key);
    }

    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)?;
        Ok(())
    }
}
