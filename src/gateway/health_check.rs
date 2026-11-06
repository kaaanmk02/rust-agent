use std::sync::Arc;
use tokio::sync::RwLock;
use crate::gateway::gateway_types::HealthCheck;

#[derive(Clone)]
pub struct AdapterHealth {
    checks: Arc<RwLock<Vec<HealthCheck>>>,
}

impl AdapterHealth {
    pub fn new() -> Self {
        Self { checks: Arc::new(RwLock::new(Vec::new())) }
    }
    pub async fn record(&self, adapter: &str, healthy: bool) {
        let mut c = self.checks.write().await;
        c.push(HealthCheck { adapter: adapter.to_string(), healthy, checked_at: chrono::Utc::now().timestamp() });
        if c.len() > 1000 {
            let drain = c.len() - 1000;
            c.drain(0..drain);
        }
    }
    pub async fn recent(&self) -> Vec<HealthCheck> {
        self.checks.read().await.clone()
    }
}
