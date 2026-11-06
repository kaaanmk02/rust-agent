use rust_agent::tools::ToolRegistry;
use std::sync::Arc;
use rust_agent::tools::builtins::EnvTool;

#[tokio::test]
async fn test_enable_disable_flow() {
    let mut reg = ToolRegistry::new();
    // disable env tool
    reg.disable_tool("env");
    assert!(reg.get("env").is_none());
    // enable it back by name
    let ok = reg.enable_tool_by_name("env");
    assert!(ok, "enable_tool_by_name should return true");
    // now should be present
    let tool = reg.get("env").expect("env tool should be present");
    // check it is the correct type
    assert_eq!(tool.name(), "env");
}
