use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use serde_json::Value;
use async_trait::async_trait;
use crate::gateway::audit::AuditLog;
use crate::gateway::conversation_state::ConversationState;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub tool_type: String,
    pub parameters: Value,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolResult {
    pub id: String,
    pub tool: String,
    pub ok: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
}

#[async_trait]
pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &str;
    fn schema(&self) -> ToolSchema;
    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult>;
}

#[derive(Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolHandler>>,
    disabled: HashSet<String>,
}



impl ToolRegistry {
    pub fn new() -> Self {
        let mut r = Self { tools: HashMap::new(), disabled: HashSet::new() };
        r.register_all();
        r
    }


    pub fn disable_tool(&mut self, name: &str) {
        self.disabled.insert(name.to_string());
        self.tools.remove(name);
    }


    // Enable a tool that was previously disabled. Returns true if the tool existed and was re‑registered.
    pub fn enable_tool_by_name(&mut self, name: &str) -> bool {
        if self.disabled.remove(name) {
            let tool: Option<Arc<dyn ToolHandler>> = match name {
                "echo" => Some(Arc::new(crate::tools::builtins::EchoTool)),
                "ping" => Some(Arc::new(crate::tools::builtins::PingTool)),
                "time" => Some(Arc::new(crate::tools::builtins::TimeTool)),
                "env" => Some(Arc::new(crate::tools::builtins::EnvTool)),
                "filestat" => Some(Arc::new(crate::tools::builtins::FileStatTool)),
                "loop" => Some(Arc::new(crate::tools::builtins::LoopTool)),
                "shell" => Some(Arc::new(crate::tools::builtins::ShellTool)),
                "file" => Some(Arc::new(crate::tools::builtins::FileTool)),
                _ => None,
            }
;
            if let Some(t) = tool {
                self.register(t);
                return true;
            }

        }

        false
    }


    // Register a tool if not disabled
    pub fn register(&mut self, t: Arc<dyn ToolHandler>) {
        let name = t.name().to_string();
        if !self.disabled.contains(&name) {
            self.tools.insert(name, t);
        }
    }





    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolHandler>> {
        self.tools.get(name).cloned()
    }


    pub async fn list(&self) -> Vec<ToolSchema> {
        self.tools.values().map(|t| t.schema()).collect()
    }


    pub async fn execute(&self, name: &str, args: Value, audit: &AuditLog, conv: &ConversationState, conv_id: &str) -> anyhow::Result<ToolResult> {
        if let Some(tool) = self.tools.get(name) {
            let res = tool.execute(args.clone()).await;
            let ok = res.as_ref().map(|r| r.ok).unwrap_or(false);
            audit.record(name, "tool_execute", None, None, serde_json::json!({"ok": ok})).await;
            let _ = conv.add_message(conv_id, "tool", &serde_json::to_string(&serde_json::json!({"tool": name, "ok": ok})).unwrap_or_default()).await;
            res
        }
 else {
            anyhow::bail!("tool not found: {}", name)
        }

    }


    fn register_all(&mut self) {
        use crate::tools::builtins::{EchoTool, PingTool, TimeTool, EnvTool, FileStatTool, LoopTool, ShellTool, FileTool};
        let tools: Vec<Arc<dyn ToolHandler>> = vec![
            Arc::new(EchoTool),
            Arc::new(PingTool),
            Arc::new(TimeTool),
            Arc::new(EnvTool),
            Arc::new(FileStatTool),
            Arc::new(LoopTool),
            Arc::new(ShellTool),
            Arc::new(FileTool),
        ];
        for t in tools {
            self.register(t);
        }

    }

}

pub mod builtins;
