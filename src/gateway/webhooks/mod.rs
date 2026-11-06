use axum::{http::StatusCode, Json};
use serde_json::Value;
use tracing::warn;

pub async fn handle_telegram(Json(body): Json<Value>) -> StatusCode {
    let channel = body.get("channel").and_then(|v| v.as_str()).unwrap_or("telegram");
    let text = body.get("text").and_then(|v| v.as_str()).unwrap_or("");
    warn!(channel=%channel, text_len=%text.len(), "telegram webhook");
    StatusCode::OK
}
pub async fn handle_slack(Json(body): Json<Value>) -> StatusCode {
    warn!(event=%body, "slack webhook");
    StatusCode::OK
}
pub async fn handle_discord(Json(body): Json<Value>) -> StatusCode {
    warn!(event=%body, "discord webhook");
    StatusCode::OK
}
pub async fn handle_whatsapp(Json(body): Json<Value>) -> StatusCode {
    warn!(event=%body, "whatsapp webhook");
    StatusCode::OK
}
