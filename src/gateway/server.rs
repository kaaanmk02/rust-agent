use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Router, Json,
};
use tower_http::cors::CorsLayer;
use tracing::info;
use crate::tools::ToolRegistry;
use crate::gateway::{
    gateway_config::GatewayConfig,
    gateway_types::{Adapter, GatewayClient, OutgoingMessage, DeliveryReceipt},
    middleware::auth_middleware,
    ws::ws_handler,
    multi_agent::RouterAgent,
    audit::AuditLog,
    conversation_state::ConversationState,
};
use crate::gateway::config_hot_reload::ConfigWatcher;
use crate::queue::MemoryQueue;
use crate::gateway::security::dangerous_tools::DangerousToolPolicy;
use crate::graceful::GracefulShutdown;
use crate::gateway::timeout_middleware::request_timeout;


#[derive(Clone)]
pub struct GatewayState {
    pub cfg: Arc<GatewayConfig>,
    pub client: GatewayClient,
    pub adapters: Arc<Vec<Box<dyn Adapter + Send + Sync>>>,
    pub tx: tokio::sync::broadcast::Sender<String>,
    pub tools: ToolRegistry,
    pub queue: Option<Arc<dyn crate::queue::QueueBackend + Send + Sync>>,
    pub audit: AuditLog,
    pub conversations: ConversationState,
    pub router: RouterAgent,
    pub danger: DangerousToolPolicy,
    pub shutdown: GracefulShutdown,
}

async fn openapi_json() -> String {
    std::fs::read_to_string("docs/openapi.json")
        .or_else(|_| std::fs::read_to_string("docs/openapi.yaml"))
        .unwrap_or_else(|_| r#"{"openapi":"3.0.0","info":{"title":"rust-agent","version":"0.1.0"},"paths":{},"components":{"schemas":{}}}"#.into())
}

pub fn build_router(state: GatewayState) -> Router {
    let state = state.clone();
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/version", get(version_handler))
        .route("/ws", get(ws_handler))
        .route("/api/chat", post(api_chat))
        .route("/api/tools", get(api_tools))
        .route("/api/tools/execute", post(api_tool_execute))
        .route("/channels", get(list_channels))
        .route("/api/gateway/config", post(update_config))
        .route("/metrics", get(metrics_handler))
        .route("/audit", get(audit_recent))
        .route("/webhook/telegram", post(crate::gateway::webhooks::handle_telegram))
        .route("/webhook/slack", post(crate::gateway::webhooks::handle_slack))
        .route("/webhook/discord", post(crate::gateway::webhooks::handle_discord))
        .route("/webhook/whatsapp", post(crate::gateway::webhooks::handle_whatsapp))
        .route("/health/adapters", get(adapter_health))
        .route("/shutdown", post(shutdown_handler))
        .route("/docs", get(docs_handler))
        .route("/openapi.json", get(openapi_json))
        // .merge is handled by Into<Router>
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(|req, next| request_timeout(std::time::Duration::from_secs(30), req, next)))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state)
}

pub async fn serve(cfg: GatewayConfig) -> anyhow::Result<()> {
    let (tx, _rx) = tokio::sync::broadcast::channel(1024);
    let client = GatewayClient::new(format!("http://{}", cfg.listen), cfg.auth_token.clone());
    let adapters = Arc::new(load_adapters(&cfg));
    let _queue = MemoryQueue::new(1024);
    let cfg_arc = Arc::new(cfg.clone());
    let tools = ToolRegistry::new();
    let router = RouterAgent::new(tools.clone());
    let queue_backend: Option<Arc<dyn crate::queue::QueueBackend + Send + Sync>> = if let Some(backend) = &cfg_arc.queue {
        if backend.backend == "persistent" {
            if let Some(path) = &backend.path {
                crate::queue::persistent::PersistentQueue::new(path.as_path())
                    .ok()
                    .map(|q| Arc::new(q) as Arc<dyn crate::queue::QueueBackend + Send + Sync>)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    let queue_mem = MemoryQueue::new(1024);
    let state = GatewayState {
        cfg: cfg_arc.clone(),
        client,
        adapters: adapters.clone(),
        tx: tx.clone(),
        tools: tools.clone(),
        queue: queue_backend.or_else(|| Some(Arc::new(queue_mem.0))),
        audit: AuditLog::new(),
        conversations: ConversationState::new(),
        router,
        danger: DangerousToolPolicy::new(),
        shutdown: GracefulShutdown::new(),
    };
    let app = build_router(state.clone());
    let addr = state.cfg.listen.parse::<std::net::SocketAddr>()?;
    info!(address = %addr, "gateway listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let tick_handle = tokio::spawn(async move {
        loop {
            for a in adapters.iter() {
                if let Ok(envelopes) = a.poll_updates().await {
                    for e in envelopes {
                        let _ = tx.send(serde_json::to_string(&e).unwrap_or_default());
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    let reload_handle = tokio::spawn(async move {
        use std::path::PathBuf;
        let _ = ConfigWatcher::new(PathBuf::from("gateway.json"), tokio::sync::RwLock::new((*cfg_arc).clone())).watch().await;
    });
    let serve_handle = tokio::spawn(async move { axum::serve(listener, app).await });
    tokio::select! {
        r = serve_handle => r??,
        _ = tick_handle => {}
    }
    let _ = state.shutdown.tx.send(());
    tokio::select! {
        _ = reload_handle => {}
        _ = tokio::signal::ctrl_c() => { tracing::info!("shutdown signal"); }
    }
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
                last_update_id: std::sync::Arc::new(std::sync::Mutex::new(t.last_update_id.unwrap_or_default())),
            }));
        }
    }
    if let Some(d) = &cfg.discord {
        if d.enabled {
            out.push(Box::new(crate::gateway::adapters::DiscordAdapter {
                token: d.bot_token.clone(),
                application_id: d.application_id.clone(),
                channel_id: d.channel_id.clone(),
                client: reqwest::Client::new(),
            }));
        }
    }
    if let Some(s) = &cfg.slack {
        if s.enabled {
            out.push(Box::new(crate::gateway::adapters::SlackAdapter {
                bot_token: s.bot_token.clone(),
                channel_id: s.channel_id.clone(),
                client: reqwest::Client::new(),
            }));
        }
    }
    if let Some(w) = &cfg.whatsapp {
        if w.enabled {
            out.push(Box::new(crate::gateway::adapters::WhatsAppAdapter {
                session_name: w.session_name.clone(),
                endpoint: w.endpoint.clone(),
                api_key: w.api_key.clone(),
                client: reqwest::Client::new(),
            }));
        }
    }
    out
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct ChatRequest {
    pub channel: String,
    pub chat_id: String,
    pub text: String,
}

async fn health_handler() -> &'static str { "ok" }

async fn api_chat(
    State(state): State<GatewayState>,
    Json(req): Json<ChatRequest>,
) -> Json<serde_json::Value> {
    let conv = state.conversations.create(&req.channel, &req.chat_id).await;
    let _ = state.conversations.add_message(&conv.id, "user", &req.text).await;
    state.audit.record("chat_message", &req.chat_id, Some(&req.channel), Some(&req.chat_id), serde_json::json!({"text": req.text})).await;
    let message = OutgoingMessage::Text { text: req.text.clone() };
    let mut receipt: Option<DeliveryReceipt> = None;
    for adapter in state.adapters.iter() {
        if adapter.name() == req.channel {
            receipt = adapter.send(&req.chat_id, message).await.ok();
            break;
        }
    }
    let _ = state.tx.send(serde_json::json!({"channel": req.channel, "chat_id": req.chat_id, "receipt": receipt}).to_string());
    Json(serde_json::json!({"ok": receipt.is_some(), "receipt": receipt}))
}

async fn api_tools(State(state): State<GatewayState>) -> Json<serde_json::Value> {
    let tools = state.tools.list().await;
    Json(serde_json::json!({"tools": tools}))
}

async fn list_channels(State(state): State<GatewayState>) -> Json<serde_json::Value> {
    let names: Vec<String> = state.adapters.iter().map(|a| a.name().to_string()).collect();
    Json(serde_json::json!({"channels": names}))
}

async fn update_config(State(state): State<GatewayState>, Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    if let Ok(bytes) = serde_json::to_vec_pretty(&body) {
        let tmp = format!("{}.tmp", state.cfg.listen);
        if let Ok(mut f) = std::fs::File::create(&tmp) {
            use std::io::Write;
            let _ = f.write_all(&bytes);
            let _ = std::fs::rename(&tmp, "gateway.json");
        }
    }
    Json(serde_json::json!({"ok": true}))
}

async fn ready_handler(State(state): State<GatewayState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ready": !state.adapters.is_empty()}))
}

async fn version_handler(State(_state): State<GatewayState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"version": env!("CARGO_PKG_VERSION"), "name": env!("CARGO_PKG_NAME")}))
}

async fn metrics_handler(State(_state): State<GatewayState>) -> String {
    crate::metrics::metrics_text()
}

async fn audit_recent(State(state): State<GatewayState>) -> Json<serde_json::Value> {
    let entries = state.audit.recent(50).await;
    Json(serde_json::json!({"entries": entries}))
}

async fn adapter_health(State(state): State<GatewayState>) -> Json<serde_json::Value> {
    let mut results = Vec::new();
    for a in state.adapters.iter() {
        let ok = a.poll_updates().await.is_ok();
        results.push(serde_json::json!({"adapter": a.name(), "ok": ok}));
    }
    Json(serde_json::json!({"adapter_health": results}))
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct ToolExecRequest {
    pub tool: String,
    pub args: serde_json::Value,
    pub channel: String,
    pub chat_id: String,
    pub confirm: Option<bool>,
}

async fn api_tool_execute(State(state): State<GatewayState>, Json(req): Json<ToolExecRequest>) -> Json<serde_json::Value> {
    let tool_opt = state.tools.get(&req.tool);
    if tool_opt.is_none() {
        return Json(serde_json::json!({"ok": false, "error": "unknown tool"}));
    }
    let schema = tool_opt.unwrap().schema();
    let is_dangerous = state.danger.is_dangerous_by_name(&schema.name).await || state.danger.is_dangerous_by_name(&req.tool).await;
    if is_dangerous && req.confirm != Some(true) {
        return Json(serde_json::json!({"ok": false, "requires_confirm": true, "message": "dangerous tool; operator approval required: set confirm=true"}));
    }
    let conv_id = format!("{}:{}", req.channel, req.chat_id);
    match state.tools.execute(&req.tool, req.args, &state.audit, &state.conversations, &conv_id).await {
        Ok(result) => Json(serde_json::json!({"ok": result.ok, "result": result.data, "error": result.error})),
        Err(e) => Json(serde_json::json!({"ok": false, "error": e.to_string()})),
    }
}

async fn shutdown_handler(State(_state): State<GatewayState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "message": "shutdown initiated"}))
}

async fn docs_handler() -> axum::response::Redirect {
    axum::response::Redirect::temporary("/openapi.json")
}

pub async fn shutdown() {
    info!("shutdown signal received");
}


