use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::tools::ToolRegistry;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub model: String,
    pub tools: Vec<String>,
    pub max_rounds: u32,
}

pub struct RouterAgent {
    profiles: Arc<RwLock<HashMap<String, AgentProfile>>>,
    tools: ToolRegistry,
}

impl RouterAgent {
    pub fn new(tools: ToolRegistry) -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("default".into(), AgentProfile { id: "default".into(), name: "default".into(), model: "default".into(), tools: vec![], max_rounds: 4 });
        Self { profiles: Arc::new(RwLock::new(profiles)), tools }
    }
    pub async fn register(&self, profile: AgentProfile) {
        self.profiles.write().await.insert(profile.id.clone(), profile);
    }
    pub async fn route(&self, channel: &str, _text: &str) -> anyhow::Result<AgentProfile> {
        let profiles = self.profiles.read().await;
        Ok(profiles.get(channel).or_else(|| profiles.get("default")).cloned().unwrap())
    }
}

impl Clone for RouterAgent {
    fn clone(&self) -> Self {
        Self {
            profiles: self.profiles.clone(),
            tools: self.tools.clone(),
        }
    }
}

