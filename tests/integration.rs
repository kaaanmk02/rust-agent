use rust_agent::gateway::server::build_router;
use rust_agent::tools::ToolRegistry;
use rust_agent::gateway::audit::AuditLog;
use rust_agent::gateway::conversation_state::ConversationState;
use rust_agent::gateway::security::dangerous_tools::DangerousToolPolicy;
use rust_agent::gateway::multi_agent::RouterAgent;
use rust_agent::graceful::GracefulShutdown;
use rust_agent::queue::MemoryQueue;

use axum::Router;
use serde_json::json;
use tower::Service;

async fn app() -> Router {
    let cfg = rust_agent::gateway::GatewayConfig {
        enabled: true,
        listen: "127.0.0.1:0".into(),
        auth_token: Some("test-token".into()),
        telegram: None,
        whatsapp: None,
        discord: None,
        slack: None,
        mcp: None,
        queue: None,
    };
    let (tx, _rx) = tokio::sync::broadcast::channel(128);
    let client = rust_agent::gateway::gateway_types::GatewayClient::new(
        format!("http://{}", cfg.listen),
        cfg.auth_token.clone(),
    );
    let state = rust_agent::gateway::server::GatewayState {
        cfg: std::sync::Arc::new(cfg.clone()),
        client,
        adapters: std::sync::Arc::new(vec![]),
        tx,
        tools: ToolRegistry::new(),
        queue: Some(std::sync::Arc::new(MemoryQueue::new(128).0)),
        audit: AuditLog::new(),
        conversations: ConversationState::new(),
        router: RouterAgent::new(ToolRegistry::new()),
        danger: DangerousToolPolicy::new(),
        shutdown: GracefulShutdown::new(),
    };
    build_router(state)
}

#[tokio::test]
async fn gateway_health_ok() {
    let mut service = app().await;
    let response = service
        .call(
            axum::http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn gateway_ready_without_adapters() {
    let mut service = app().await;
    let response = service
        .call(
            axum::http::Request::builder()
                .uri("/ready")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed.get("ready").and_then(|v| v.as_bool()), Some(false));
}

#[tokio::test]
async fn gateway_version_ok() {
    let mut service = app().await;
    let response = service
        .call(
            axum::http::Request::builder()
                .uri("/version")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed.get("name").and_then(|v| v.as_str()), Some("rust-agent"));
}

#[tokio::test]
async fn gateway_openapi_requires_auth() {
    let mut service = app().await;
    let response = service
        .call(
            axum::http::Request::builder()
                .uri("/openapi.json")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn gateway_api_chat_without_token_unauthorized() {
    let mut service = app().await;
    let body = json!({"channel": "cli", "chat_id": "1", "text": "hello"});
    let response = service
        .call(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/chat")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn gateway_api_chat_with_token_no_adapter() {
    let mut service = app().await;
    let body = json!({"channel": "cli", "chat_id": "1", "text": "hello"});
    let bytes = serde_json::to_vec(&body).unwrap();
    let response = service
        .call(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/chat")
                .header("content-type", "application/json")
                .header("authorization", "Bearer test-token")
                .body(axum::body::Body::from(bytes))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed.get("ok").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(parsed.get("receipt").and_then(|v| v.as_object()), None);
}

#[tokio::test]
async fn gateway_api_tools_execute_echo() {
    let mut service = app().await;
    let body = json!({"tool": "echo", "args": {"text": "selam"}, "channel": "cli", "chat_id": "1"});
    let bytes = serde_json::to_vec(&body).unwrap();
    let response = service
        .call(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/tools/execute")
                .header("content-type", "application/json")
                .header("authorization", "Bearer test-token")
                .body(axum::body::Body::from(bytes))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed.get("ok").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(parsed.get("result").and_then(|v| v.get("text")).and_then(|v| v.as_str()), Some("selam"));
}

#[tokio::test]
async fn gateway_api_gateway_config_get_method_not_allowed() {
    let mut service = app().await;
    let response = service
        .call(
            axum::http::Request::builder()
                .uri("/api/gateway/config")
                .header("authorization", "Bearer test-token")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 405);
}

#[tokio::test]
async fn gateway_api_gateway_config_post() {
    let mut service = app().await;
    let body = json!({"listen": "127.0.0.1:19999"});
    let bytes = serde_json::to_vec(&body).unwrap();
    let response = service
        .call(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/gateway/config")
                .header("content-type", "application/json")
                .header("authorization", "Bearer test-token")
                .body(axum::body::Body::from(bytes))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed.get("ok").and_then(|v| v.as_bool()), Some(true));
}
