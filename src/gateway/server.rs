use std::sync::Arc;
use axum::{
    extract::{State, ws::{WebSocketUpgrade, Message}},
    middleware,
    routing::{get, post},
    Router, Json,
};
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::gateway::{
    gateway_types::{GatewayClient, Adapter, OutgoingMessage, DeliveryReceipt},
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
    let adapters = load_adapters(&cfg);
    let state = GatewayState {
        cfg: Arc::new(cfg),
        client,
        adapters: Arc::new(adapters),
        tx,
    };
    let app = build_router(state);
    let addr = "0.0.0.0:9090".parse::<std::net::SocketAddr>().unwrap();
    info!(address = %addr, "gateway listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn load_adapters(cfg: &GatewayConfig) -> Vec<Box<dyn Adapter + Send + Sync>> {
    let mut out: Vec<Box<dyn Adapter + Send + Sync>> = Vec::new();
    if let Some(t) = &cfg.telegram {
        if t.enabled {
            out.push(Box::new(crate::gateway::adapters::TelegramAdapter {
                token: t.bot_token.clone(),
                allowed_user_ids: t.allowed_user_ids.clone(),
                client: reqwest::Client::new(),
            }));
        }
    }
    if let Some(d) = &cfg.discord {
        if d.enabled {
            out.push(Box::new(crate::gateway::adapters::DiscordAdapter {
                token: d.bot_token.clone(),
                application_id: d.application_id.clone(),
                client: reqwest::Client::new(),
            }));
        }
    }
    if let Some(s) = &cfg.slack {
        if s.enabled {
            out.push(Box::new(crate::gateway::adapters::SlackAdapter {
                bot_token: s.bot_token.clone(),
                client: reqwest::Client::new(),
            }));
        }
    }
    if let Some(w) = &cfg.whatsapp {
        if w.enabled {
            out.push(Box::new(crate::gateway::adapters::WhatsAppAdapter {
                endpoint: w.endpoint.clone(),
                api_key: w.api_key.clone(),
                client: reqwest::Client::new(),
            }));
        }
    }
    out
}

#[derive(serde::Deserialize)]
pub struct ChatRequest {
    pub channel: String,
    pub chat_id: String,
    pub text: String,
}

async fn api_chat(
    State(state): State<GatewayState>,
    Json(req): Json<ChatRequest>,
) -> Json<serde_json::Value> {
    let message = OutgoingMessage::Text { text: req.text };
    let mut receipt: Option<DeliveryReceipt> = None;
    for adapter in state.adapters.iter() {
        if adapter.name() == req.channel {
            receipt = adapter.send(&req.chat_id, message).await.ok();
            break;
        }
    }
    Json(serde_json::json!({"ok": receipt.is_some(), "receipt": receipt}))
}

async fn api_tools(
    State(_state): State<GatewayState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"tools": []}))
}

async fn list_channels(
    State(state): State<GatewayState>,
) -> Json<serde_json::Value> {
    let names: Vec<String> = state.adapters.iter().map(|a| a.name().to_string()).collect();
    Json(serde_json::json!({"channels": names}))
}

async fn update_config(
    State(_state): State<GatewayState>,
    Json(_body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}
