use crate::tools::{ToolHandler, ToolResult, ToolSchema};
use async_trait::async_trait;
use serde_json::Value;
use std::fs;
use std::path::Path;

// Basic tools
pub struct EchoTool;
pub struct PingTool;
pub struct TimeTool;
pub struct EnvTool;
pub struct FileStatTool;
pub struct LoopTool;
pub struct ShellTool;
pub struct FileTool;

#[async_trait]
impl ToolHandler for EchoTool {
    fn name(&self) -> &str { "echo" }
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "echo".into(),
            description: "Echo back the input text".into(),
            tool_type: "function".into(),
            parameters: serde_json::json!({"type":"object","properties":{"text":{"type":"string"}},"required":["text"]}),
            required: vec!["text".into()],
        }
    }
    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
        Ok(ToolResult { id: String::new(), tool: "echo".into(), ok: true, data: Some(serde_json::json!({"text": text})), error: None })
    }
}

#[async_trait]
impl ToolHandler for PingTool {
    fn name(&self) -> &str { "ping" }
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "ping".into(),
            description: "Health check tool".into(),
            tool_type: "function".into(),
            parameters: serde_json::json!({"type":"object","properties":{}}),
            required: vec![],
        }
    }
    async fn execute(&self, _args: Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { id: String::new(), tool: "ping".into(), ok: true, data: Some(serde_json::json!({"pong": true})), error: None })
    }
}

#[async_trait]
impl ToolHandler for TimeTool {
    fn name(&self) -> &str { "time" }
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "time".into(),
            description: "Return current UTC timestamp".into(),
            tool_type: "function".into(),
            parameters: serde_json::json!({"type":"object","properties":{}}),
            required: vec![],
        }
    }
    async fn execute(&self, _args: Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { id: String::new(), tool: "time".into(), ok: true, data: Some(serde_json::json!({"ts": chrono::Utc::now().timestamp()})), error: None })
    }
}

#[async_trait]
impl ToolHandler for EnvTool {
    fn name(&self) -> &str { "env" }
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "env".into(),
            description: "Read allowed environment variables".into(),
            tool_type: "function".into(),
            parameters: serde_json::json!({"type":"object","properties":{"var":{"type":"string"}},"required":["var"]}),
            required: vec!["var".into()],
        }
    }
    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let var = args.get("var").and_then(|v| v.as_str()).unwrap_or("");
        let allowed = ["HOME", "PATH", "USER", "SHELL"];
        if !allowed.contains(&var) {
            return Ok(ToolResult { id: String::new(), tool: "env".into(), ok: false, data: None, error: Some(format!("variable {} not allowed", var)) });
        }
        match std::env::var(var) {
            Ok(val) => Ok(ToolResult { id: String::new(), tool: "env".into(), ok: true, data: Some(serde_json::json!({"value": val})), error: None }),
            Err(e) => Ok(ToolResult { id: String::new(), tool: "env".into(), ok: false, data: None, error: Some(e.to_string()) }),
        }
    }
}

#[async_trait]
impl ToolHandler for FileStatTool {
    fn name(&self) -> &str { "filestat" }
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "filestat".into(),
            description: "Return file metadata (size, modified) within sandbox".into(),
            tool_type: "function".into(),
            parameters: serde_json::json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]}),
            required: vec!["path".into()],
        }
    }
    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let base = Path::new("/tmp/rust-agent-files");
        let target = base.join(path);
        let target_str = target.to_string_lossy().to_string();
        if target_str.contains("..") || !target_str.starts_with(base.to_string_lossy().as_ref()) {
            return Ok(ToolResult { id: String::new(), tool: "filestat".into(), ok: false, data: None, error: Some("path escapes sandbox".into()) });
        }
        match fs::metadata(&target) {
            Ok(meta) => {
                let size = meta.len();
                let modified = meta.modified().ok().and_then(|t| t.elapsed().ok()).map(|e| e.as_secs());
                Ok(ToolResult { id: String::new(), tool: "filestat".into(), ok: true, data: Some(serde_json::json!({"size": size, "modified_seconds_ago": modified})), error: None })
            }
            Err(e) => Ok(ToolResult { id: String::new(), tool: "filestat".into(), ok: false, data: None, error: Some(e.to_string()) }),
        }
    }
}

#[async_trait]
impl ToolHandler for LoopTool {
    fn name(&self) -> &str { "loop" }
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "loop".into(),
            description: "Runs a simple countdown loop and returns final value".into(),
            tool_type: "function".into(),
            parameters: serde_json::json!({"type":"object","properties":{"times":{"type":"integer","minimum":1}},"required":["times"]}),
            required: vec!["times".into()],
        }
    }
    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let times = args.get("times").and_then(|v| v.as_i64()).unwrap_or(1);
        let mut counter = times;
        while counter > 0 { counter -= 1; }
        Ok(ToolResult { id: String::new(), tool: "loop".into(), ok: true, data: Some(serde_json::json!({"result": counter})), error: None })
    }
}

#[async_trait]
impl ToolHandler for ShellTool {
    fn name(&self) -> &str { "shell" }
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "shell".into(),
            description: "Run an allowed shell command".into(),
            tool_type: "function".into(),
            parameters: serde_json::json!({"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}),
            required: vec!["command".into()],
        }
    }
    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
        let blocked_prefixes = ["rm -rf", "sudo", "su ", "dd ", ":(){", "chmod +x"];
        let lower = command.to_lowercase();
        let blocked = blocked_prefixes.iter().any(|p| lower.starts_with(p));
        if blocked || command.trim().is_empty() {
            return Ok(ToolResult { id: String::new(), tool: "shell".into(), ok: false, data: None, error: Some("blocked or empty command".into()) });
        }
        let output = tokio::process::Command::new("sh").arg("-c").arg(command).output().await;
        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                Ok(ToolResult { id: String::new(), tool: "shell".into(), ok: o.status.success(), data: Some(serde_json::json!({"stdout": stdout, "stderr": stderr, "code": o.status.code()})), error: if o.status.success() { None } else { Some(format!("exit {:?}", o.status.code())) } })
            }
            Err(e) => Ok(ToolResult { id: String::new(), tool: "shell".into(), ok: false, data: None, error: Some(e.to_string()) }),
        }
    }
}

#[async_trait]
impl ToolHandler for FileTool {
    fn name(&self) -> &str { "file" }
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "file".into(),
            description: "Safe file read/write under sandboxed dir".into(),
            tool_type: "function".into(),
            parameters: serde_json::json!({"type":"object","properties":{"action":{"type":"string","enum":["read","write"]},"path":{"type":"string"},"content":{"type":"string"}},"required":["action","path"]}),
            required: vec!["action".into(), "path".into()],
        }
    }
    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("read");
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let base = Path::new("/tmp/rust-agent-files");
        let target = base.join(path);
        let target_str = target.to_string_lossy().to_string();
        if target_str.contains("..") || !target_str.starts_with(base.to_string_lossy().as_ref()) {
            return Ok(ToolResult { id: String::new(), tool: "file".into(), ok: false, data: None, error: Some("path escapes sandbox".into()) });
        }
        match action {
            "read" => {
                if !target.exists() {
                    return Ok(ToolResult { id: String::new(), tool: "file".into(), ok: false, data: None, error: Some("not found".into()) });
                }
                match fs::read_to_string(&target) {
                    Ok(content) => Ok(ToolResult { id: String::new(), tool: "file".into(), ok: true, data: Some(serde_json::json!({"content": content})), error: None }),
                    Err(e) => Ok(ToolResult { id: String::new(), tool: "file".into(), ok: false, data: None, error: Some(e.to_string()) }),
                }
            }
            "write" => {
                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                if let Some(parent) = target.parent() { let _ = fs::create_dir_all(parent); }
                match fs::write(&target, content) {
                    Ok(_) => Ok(ToolResult { id: String::new(), tool: "file".into(), ok: true, data: Some(serde_json::json!({"path": target_str, "bytes": content.len()})), error: None }),
                    Err(e) => Ok(ToolResult { id: String::new(), tool: "file".into(), ok: false, data: None, error: Some(e.to_string()) }),
                }
            }
            other => Ok(ToolResult { id: String::new(), tool: "file".into(), ok: false, data: None, error: Some(format!("unsupported action: {}", other)) }),
        }
    }
}
