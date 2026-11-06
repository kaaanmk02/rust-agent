use crate::gateway::gateway_types::{Adapter, ChannelEnvelope, DeliveryReceipt, IncomingMessage, OutgoingMessage};
use crate::mcp::McpClient;
use anyhow::Result;
use async_trait::async_trait;
use tracing::warn;

pub struct McpAdapter {
    pub client: McpClient,
    pub chat_id: String,
}

impl McpAdapter {
    pub fn new(client: McpClient, chat_id: impl Into<String>) -> Self {
        Self { client, chat_id: chat_id.into() }
    }
}

#[async_trait]
impl Adapter for McpAdapter {
    fn name(&self) -> &str {
        "mcp"
    }

    async fn send(&self, _chat_id: &str, message: OutgoingMessage) -> Result<DeliveryReceipt> {
        let text = match message {
            OutgoingMessage::Text { text } => text,
            _ => String::new(),
        };
        if text.is_empty() {
            return Ok(DeliveryReceipt { id: String::new(), channel: self.name().to_string(), ok: false, error: Some("empty message".into()) });
        }
        match self.client.call("message/send", serde_json::json!({"chat_id": self.chat_id, "text": text})).await {
            Ok(v) => {
                let id = v.get("id").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                Ok(DeliveryReceipt { id, channel: self.name().to_string(), ok: true, error: None })
            }
            Err(e) => {
                warn!(error = %e, "mcp send failed");
                Ok(DeliveryReceipt { id: String::new(), channel: self.name().to_string(), ok: false, error: Some(e.to_string()) })
            }
        }
    }

    async fn poll_updates(&self) -> Result<Vec<ChannelEnvelope>> {
        match self.client.call("updates/poll", serde_json::json!({"chat_id": self.chat_id})).await {
            Ok(v) => {
                let mut out = Vec::new();
                if let Some(arr) = v.get("updates").and_then(|u| u.as_array()) {
                    for item in arr {
                        out.push(ChannelEnvelope {
                            channel: self.name().to_string(),
                            chat_id: self.chat_id.clone(),
                            from: item.get("from").and_then(|f| f.as_str()).map(|s| s.to_string()),
                            message: IncomingMessage::Text { text: item.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string() },
                            raw: Some(item.clone()),
                        });
                    }
                }
                Ok(out)
            }
            Err(e) => {
                warn!(error = %e, "mcp poll failed");
                Ok(Vec::new())
            }
        }
    }
}
