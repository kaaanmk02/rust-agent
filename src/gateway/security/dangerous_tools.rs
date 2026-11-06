use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct DangerousToolPolicy {
    allowed: Arc<RwLock<HashSet<String>>>,
}

impl DangerousToolPolicy {
    pub fn new() -> Self {
        let mut set = HashSet::new();
        for name in ["exec","system","eval","bash","shell","cmd","powershell"] {
            set.insert(name.into());
        }
        Self { allowed: Arc::new(RwLock::new(set)) }
    }

    pub async fn is_dangerous(&self, tool: &str) -> bool {
        self.allowed.read().await.contains(tool)
    }

    pub async fn is_dangerous_by_name(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        self.allowed.read().await.iter().any(|d| lower.contains(d.as_str()))
    }

    pub async fn allow(&self, tool: &str, allowed: bool) {
        let mut guard = self.allowed.write().await;
        if allowed { guard.insert(tool.into()); }
        else { guard.remove(tool); }
    }
}
