pub mod runner;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentTask {
    pub id: String,
    pub goal: String,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentResult {
    pub id: String,
    pub summary: String,
    pub success: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DelegationManager {
    pub tasks: Vec<SubagentTask>,
}

impl DelegationManager {
    pub fn spawn(&mut self, task: SubagentTask) {
        self.tasks.push(task);
    }

    pub fn list(&self) -> &[SubagentTask] {
        &self.tasks
    }
}
