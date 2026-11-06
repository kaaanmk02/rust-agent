use rust_agent::gateway::GatewayConfig;
use std::fs;

#[test]
fn gateway_config_roundtrip_json() {
    let path = std::env::temp_dir().join("rust-agent-config-roundtrip.json");
    let original = GatewayConfig {
        enabled: true,
        listen: "127.0.0.1:19999".into(),
        auth_token: Some("tok".into()),
        telegram: None,
        whatsapp: None,
        discord: None,
        slack: None,
        mcp: None,
        queue: None,
    };
    let data = serde_json::to_string_pretty(&original).unwrap();
    fs::write(&path, data).unwrap();
    let read_back: GatewayConfig = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(read_back.listen, original.listen);
    assert_eq!(read_back.auth_token, original.auth_token);
    let _ = fs::remove_file(path);
}

#[test]
fn gateway_config_default_unchanged_after_roundtrip() {
    let path = std::env::temp_dir().join("rust-agent-config-default.json");
    let original = GatewayConfig::default();
    let data = serde_json::to_string_pretty(&original).unwrap();
    fs::write(&path, data).unwrap();
    let read_back: GatewayConfig = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(read_back.listen, original.listen);
    assert_eq!(read_back.enabled, original.enabled);
    let _ = fs::remove_file(path);
}
