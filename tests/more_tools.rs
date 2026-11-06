use rust_agent::tools::ToolRegistry;

#[tokio::test]
async fn builtin_env_reads_allowed() {
    let registry = ToolRegistry::new();
    let tool = registry.get("env").expect("env registered");
    let result = tool.execute(serde_json::json!({"var": "HOME"})).await.unwrap();
    assert!(result.ok, "env HOME failed: {:?}", result.error);
    assert!(result.data.unwrap().get("value").is_some());
}

#[tokio::test]
async fn builtin_env_blocks_disallowed() {
    let registry = ToolRegistry::new();
    let tool = registry.get("env").expect("env registered");
    let result = tool.execute(serde_json::json!({"var": "SECRET_KEY"})).await.unwrap();
    assert!(!result.ok, "disallowed var should be blocked");
}

#[tokio::test]
async fn builtin_filestat_reads_existing() {
    let registry = ToolRegistry::new();
    let tool = registry.get("filestat").expect("filestat registered");
    let file_tool = registry.get("file").expect("file registered");
    let _ = file_tool.execute(serde_json::json!({"action":"write","path":"temp.txt","content":"data"})).await.unwrap();
    let result = tool.execute(serde_json::json!({"path":"temp.txt"})).await.unwrap();
    assert!(result.ok, "filestat failed: {:?}", result.error);
    let data = result.data.unwrap();
    assert!(data.get("size").is_some());
}

#[tokio::test]
async fn builtin_loop_counts_down() {
    let registry = ToolRegistry::new();
    let tool = registry.get("loop").expect("loop registered");
    let result = tool.execute(serde_json::json!({"times": 3})).await.unwrap();
    assert!(result.ok);
    let val = result.data.unwrap().get("result").and_then(|v| v.as_i64()).unwrap_or(-1);
    assert_eq!(val, 0, "loop should count down to 0");
}
