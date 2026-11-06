use rust_agent::tools::ToolRegistry;

#[tokio::test]
async fn builtin_tool_schemas_valid() {
    let registry = ToolRegistry::new();
    let tools = registry.list().await;
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"echo"), "echo missing in {:?}", names);
    assert!(names.contains(&"ping"), "ping missing in {:?}", names);
    assert!(names.contains(&"time"), "time missing in {:?}", names);
    assert!(names.contains(&"shell"), "shell missing in {:?}", names);
    assert!(names.contains(&"file"), "file missing in {:?}", names);
}

#[tokio::test]
async fn builtin_echo_returns_text() {
    let registry = ToolRegistry::new();
    let tool = registry.get("echo").expect("echo registered");
    let result = tool.execute(serde_json::json!({"text": "selam"})).await.unwrap();
    assert!(result.ok, "echo failed: {:?}", result.error);
    assert_eq!(result.data.unwrap().get("text").and_then(|v| v.as_str()), Some("selam"));
}

#[tokio::test]
async fn builtin_ping_returns_pong() {
    let registry = ToolRegistry::new();
    let tool = registry.get("ping").expect("ping registered");
    let result = tool.execute(serde_json::json!({})).await.unwrap();
    assert!(result.ok, "ping failed: {:?}", result.error);
    assert_eq!(result.data.unwrap().get("pong").and_then(|v| v.as_bool()), Some(true));
}

#[tokio::test]
async fn builtin_time_returns_timestamp() {
    let registry = ToolRegistry::new();
    let tool = registry.get("time").expect("time registered");
    let result = tool.execute(serde_json::json!({})).await.unwrap();
    assert!(result.ok, "time failed: {:?}", result.error);
    assert!(result.data.unwrap().get("ts").and_then(|v| v.as_i64()).is_some());
}

#[tokio::test]
async fn builtin_shell_blocks_dangerous_command() {
    let registry = ToolRegistry::new();
    let tool = registry.get("shell").expect("shell registered");
    let result = tool.execute(serde_json::json!({"command": "rm -rf /"})).await.unwrap();
    assert!(!result.ok, "dangerous command should be blocked");
}

#[tokio::test]
async fn builtin_shell_runs_safe_command() {
    let registry = ToolRegistry::new();
    let tool = registry.get("shell").expect("shell registered");
    let result = tool.execute(serde_json::json!({"command": "echo hello"})).await.unwrap();
    assert!(result.ok, "shell failed: {:?}", result.error);
    let data = result.data.unwrap();
    assert_eq!(data.get("stdout").and_then(|v| v.as_str()), Some("hello\n"));
}

#[tokio::test]
async fn builtin_file_read_write_sandbox() {
    let registry = ToolRegistry::new();
    let tool = registry.get("file").expect("file registered");
    let write = tool.execute(serde_json::json!({"action": "write", "path": "test/hello.txt", "content": "selam"})).await.unwrap();
    assert!(write.ok, "file write failed: {:?}", write.error);
    let read = tool.execute(serde_json::json!({"action": "read", "path": "test/hello.txt"})).await.unwrap();
    assert!(read.ok, "file read failed: {:?}", read.error);
    assert_eq!(read.data.unwrap().get("content").and_then(|v| v.as_str()), Some("selam"));
}

#[tokio::test]
async fn builtin_file_blocks_path_traversal() {
    let registry = ToolRegistry::new();
    let tool = registry.get("file").expect("file registered");
    let result = tool.execute(serde_json::json!({"action": "read", "path": "../../etc/passwd"})).await.unwrap();
    assert!(!result.ok, "path traversal should be blocked");
}

#[tokio::test]
async fn builtin_unknown_tool_not_found() {
    let registry = ToolRegistry::new();
    assert!(registry.get("nonexistent").is_none());
}
