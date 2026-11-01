use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncomingMessage {
    Text { text: String },
    Media { url: String, mime_type: String },
    Event { event: String, data: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutgoingMessage {
    Text { text: String },
    Media { url: String, caption: Option<String> },
    Event { event: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEnvelope {
    pub channel: String,
    pub chat_id: String,
    pub from: Option<String>,
    pub message: IncomingMessage,
    pub raw: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub id: String,
    pub channel: String,
    pub ok: bool,
    pub error: Option<String>,
}

pub trait Adapter: Send + Sync {
    fn name(&self) -> &str;
    fn send(&self, chat_id: &str, message: OutgoingMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<DeliveryReceipt>> + Send>>;
    fn poll_updates(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<ChannelEnvelope>>> + Send>>;
}

pub struct GatewayClient {
    pub base_url: String,
    pub token: Option<String>,
}

impl GatewayClient {
    pub fn new(base_url: impl Into<String>, token: Option<String>) -> Self {
        Self { base_url: base_url.into(), token }
    }
}
