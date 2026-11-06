use std::path::PathBuf;
use tokio::sync::RwLock;
use crate::gateway::GatewayConfig;

pub struct ConfigWatcher {
    path: PathBuf,
    state: RwLock<GatewayConfig>,
}

impl ConfigWatcher {
    pub fn new(path: PathBuf, state: RwLock<GatewayConfig>) -> Self {
        Self { path, state }
    }

    pub async fn watch(self) -> anyhow::Result<()> {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            if let Ok(data) = std::fs::read_to_string(&self.path) {
                if let Ok(cfg) = serde_json::from_str::<GatewayConfig>(&data) {
                    let mut s = self.state.write().await;
                    *s = cfg;
                }
            }
        }
    }
}
