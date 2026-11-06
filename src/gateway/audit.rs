use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Clone)]
pub struct AuditLog {
    entries: Arc<RwLock<Vec<AuditEntry>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub timestamp: i64,
    pub action: String,
    pub actor: String,
    pub channel: Option<String>,
    pub chat_id: Option<String>,
    pub details: serde_json::Value,
}

impl AuditLog {
    pub fn new() -> Self {
        Self { entries: Arc::new(RwLock::new(Vec::new())) }
    }

    pub async fn record(&self, action: &str, actor: &str, channel: Option<&str>, chat_id: Option<&str>, details: serde_json::Value) {
        let entry = AuditEntry {
            timestamp: chrono::Utc::now().timestamp(),
            action: action.to_string(),
            actor: actor.to_string(),
            channel: channel.map(|s| s.to_string()),
            chat_id: chat_id.map(|s| s.to_string()),
            details,
        };
        self.entries.write().await.push(entry.clone());
        warn!(action = %action, actor = %actor, channel = ?channel, chat_id = ?chat_id, "audit");
    }

    pub async fn recent(&self, limit: usize) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        entries.iter().rev().take(limit).cloned().collect()
    }
}
