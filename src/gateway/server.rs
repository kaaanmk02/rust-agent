use std::sync::Arc;
use axum::{
    extract::{State, ws::{WebSocketUpgrade, Message}},
    middleware,
    response::Response,
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::gateway::{
    gateway_types::{GatewayClient, Adapter},
    middleware::auth_middleware,
    ws::ws_handler,
    GatewayConfig,
};

#[derive(Clone)]
pub struct GatewayState {
    pub cfg: Arc<GatewayConfig>,
    pub client: GatewayClient,
    pub adapters: Arc<Vec<Box<dyn Adapter + Send + Sync>>>,
    pub tx: tokio::sync::broadcast::Sender<String>,
}

pub fn build_router(state: GatewayState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(ws_handler))
        .route("/api/chat", post(api_chat))
        .route("/api/tools", get(api_tools))
        .route("/channels", get(list_channels))
        .route("/api/gateway/config", post(update_config))
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state)
}

pub async fn serve(cfg: GatewayConfig) -> anyhow::Result<()> {
    let (tx, _rx) = tokio::sync::broadcast::channel(1024);
    let client = GatewayClient::new(format!("http://{}", cfg.listen), cfg.auth_token.clone());
    let state = GatewayState {
        cfg: Arc::new(cfg),
        client,
        adapters: Arc::new(vec![]),
        tx,
    };
    let app = build_router(state);
    let addr = "0.0.0.0:9090".parse::<std::net::SocketAddr>().unwrap();
    info!(address = %addr, "gateway listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn api_chat(
    State(_state): State<GatewayState>,
    axum::Json(_req): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"ok": true, "reply": "scaffold"}))
}

async fn api_tools(
    State(_state): State<GatewayState>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"tools": []}))
}

async fn list_channels(
    State(_state): State<GatewayState>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"channels": []}))
}

async fn update_config(
    State(_state): State<GatewayState>,
    axum::Json(_body): axum::Json<serde_json::Value>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"ok": true}))
}
