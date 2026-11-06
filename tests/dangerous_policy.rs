use rust_agent::gateway::security::dangerous_tools::DangerousToolPolicy;

#[tokio::test]
async fn dangerous_tool_policy_blocks_default() {
    let policy = DangerousToolPolicy::new();
    assert!(policy.is_dangerous("exec").await);
    assert!(policy.is_dangerous_by_name("run_exec").await);
    assert!(!policy.is_dangerous("safe_tool").await);
    // Allow a previously dangerous tool
    policy.allow("exec", false).await;
    assert!(!policy.is_dangerous("exec").await);
    // Disallow again
    policy.allow("exec", true).await;
    assert!(policy.is_dangerous("exec").await);
}
