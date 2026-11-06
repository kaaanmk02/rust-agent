use async_trait::async_trait;
use reqwest::Client;


use crate::gateway::gateway_types::{Adapter, ChannelEnvelope, DeliveryReceipt, IncomingMessage, OutgoingMessage};

#[derive(Clone)]
pub struct TelegramAdapter {
    pub token: String,
    pub allowed_user_ids: Vec<i64>,
    pub client: Client,
    pub last_update_id: std::sync::Arc<std::sync::Mutex<i64>>,
}

impl TelegramAdapter {
    fn next_offsets(&self) -> (i64, i64) {
        let guard = self.last_update_id.lock().unwrap();
        (*guard, *guard)
    }

    fn commit_offset(&self, new_offset: Option<i64>) {
        if let Some(max_id) = new_offset {
            *self.last_update_id.lock().unwrap() = max_id;
        }
    }
}

#[async_trait]
impl Adapter for TelegramAdapter {
    fn name(&self) -> &str {
        "telegram"
    }

    async fn send(&self, chat_id: &str, message: OutgoingMessage) -> anyhow::Result<DeliveryReceipt> {
        let text = match message {
            OutgoingMessage::Text { text } => text,
            OutgoingMessage::Media { caption, .. } => caption.unwrap_or_default(),
            OutgoingMessage::Event { .. } => String::new(),
        };
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        let body = serde_json::json!({"chat_id": chat_id, "text": text});
        let resp = self.client.post(url).json(&body).send().await?;
        if resp.status().is_success() {
            Ok(DeliveryReceipt { id: chat_id.into(), channel: "telegram".into(), ok: true, error: None })
        } else {
            Ok(DeliveryReceipt { id: chat_id.into(), channel: "telegram".into(), ok: false, error: Some(resp.status().to_string()) })
        }
    }

    async fn poll_updates(&self) -> anyhow::Result<Vec<ChannelEnvelope>> {
        let (base_offset, new_offset) = self.next_offsets();
        let url = format!("https://api.telegram.org/bot{}/getUpdates", self.token);
        let body = serde_json::json!({"offset": base_offset + 1, "timeout": 0});
        let resp = self.client.post(url).json(&body).send().await?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let v: serde_json::Value = resp.json().await?;
        let mut out: Vec<ChannelEnvelope> = Vec::new();
        if let Some(updates) = v.get("result").and_then(|x| x.as_array()) {
            for u in updates {
                let upd_id = u.get("update_id").and_then(|x| x.as_i64()).unwrap_or_default();
                if upd_id > new_offset {
                    out.push(parse_update(u.clone()));
                }
            }
        }
        self.commit_offset(
            out.iter()
                .map(|e| e.raw.as_ref())
                .filter_map(|r| r.and_then(|v| v.get("update_id")).and_then(|x| x.as_i64()))
                .max()
                .or(Some(new_offset)),
        );
        Ok(out)
    }
}

fn parse_update(u: serde_json::Value) -> ChannelEnvelope {
    let chat_id = u.get("message").and_then(|m| m.get("chat")).and_then(|c| c.get("id")).and_then(|x| x.as_str()).unwrap_or_default().to_string();
    let text = u.get("message").and_then(|m| m.get("text")).and_then(|x| x.as_str()).unwrap_or_default().to_string();
    let from = u.get("message").and_then(|m| m.get("from")).and_then(|f| f.get("username")).and_then(|x| x.as_str()).map(|s| s.to_string());
    ChannelEnvelope { channel: "telegram".into(), chat_id, from, message: IncomingMessage::Text { text }, raw: Some(u) }
}

#[derive(Clone)]
pub struct DiscordAdapter {
    pub token: String,
    pub application_id: String,
    pub channel_id: String,
    pub client: Client,
}

#[async_trait]
impl Adapter for DiscordAdapter {
    fn name(&self) -> &str {
        "discord"
    }

    async fn send(&self, chat_id: &str, message: OutgoingMessage) -> anyhow::Result<DeliveryReceipt> {
        let text = match message {
            OutgoingMessage::Text { text } => text,
            OutgoingMessage::Media { caption, .. } => caption.unwrap_or_default(),
            OutgoingMessage::Event { .. } => String::new(),
        };
        let url = format!("https://discord.com/api/v10/channels/{}/messages", chat_id);
        let body = serde_json::json!({"content": text});
        let resp = self.client.post(url).bearer_auth(&self.token).json(&body).send().await?;
        if resp.status().is_success() {
            Ok(DeliveryReceipt { id: chat_id.into(), channel: "discord".into(), ok: true, error: None })
        } else {
            Ok(DeliveryReceipt { id: chat_id.into(), channel: "discord".into(), ok: false, error: Some(resp.status().to_string()) })
        }
    }

    async fn poll_updates(&self) -> anyhow::Result<Vec<ChannelEnvelope>> {
        if self.channel_id.is_empty() {
            return Ok(vec![]);
        }
        let url = format!("https://discord.com/api/v10/channels/{}/messages?limit=10", self.channel_id);
        let resp = self.client.get(url).bearer_auth(&self.token).send().await?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let arr: serde_json::Value = resp.json().await?;
        let mut out = Vec::new();
        if let Some(items) = arr.as_array() {
            for item in items.iter().take(10) {
                let content = item.get("content").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                let author_id = item.get("author").and_then(|a| a.get("id")).and_then(|x| x.as_str()).map(|s| s.to_string());
                out.push(ChannelEnvelope {
                    channel: "discord".into(),
                    chat_id: self.channel_id.clone(),
                    from: author_id,
                    message: IncomingMessage::Text { text: content },
                    raw: Some(item.clone()),
                });
            }
        }
        Ok(out)
    }
}

#[derive(Clone)]
pub struct SlackAdapter {
    pub bot_token: String,
    pub channel_id: String,
    pub client: Client,
}

#[async_trait]
impl Adapter for SlackAdapter {
    fn name(&self) -> &str {
        "slack"
    }

    async fn send(&self, chat_id: &str, message: OutgoingMessage) -> anyhow::Result<DeliveryReceipt> {
        let text = match message {
            OutgoingMessage::Text { text } => text,
            OutgoingMessage::Media { caption, .. } => caption.unwrap_or_default(),
            OutgoingMessage::Event { .. } => String::new(),
        };
        let url = "https://slack.com/api/chat.postMessage";
        let body = serde_json::json!({"channel": chat_id, "text": text});
        let resp = self.client.post(url).bearer_auth(&self.bot_token).json(&body).send().await?;
        if resp.status().is_success() {
            Ok(DeliveryReceipt { id: chat_id.into(), channel: "slack".into(), ok: true, error: None })
        } else {
            Ok(DeliveryReceipt { id: chat_id.into(), channel: "slack".into(), ok: false, error: Some(resp.status().to_string()) })
        }
    }

    async fn poll_updates(&self) -> anyhow::Result<Vec<ChannelEnvelope>> {
        if self.channel_id.is_empty() {
            return Ok(vec![]);
        }
        let url = "https://slack.com/api/conversations.history";
        let body = serde_json::json!({"channel": self.channel_id, "limit": 10});
        let resp = self.client.post(url).bearer_auth(&self.bot_token).json(&body).send().await?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let val: serde_json::Value = resp.json().await?;
        let mut out = Vec::new();
        if let Some(items) = val.get("messages").and_then(|x| x.as_array()) {
            for item in items {
                let text = item.get("text").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                let user = item.get("user").and_then(|x| x.as_str()).map(|s| s.to_string());
                out.push(ChannelEnvelope {
                    channel: "slack".into(),
                    chat_id: self.channel_id.clone(),
                    from: user,
                    message: IncomingMessage::Text { text },
                    raw: Some(item.clone()),
                });
            }
        }
        Ok(out)
    }
}

#[derive(Clone)]
pub struct WhatsAppAdapter {
    pub session_name: String,
    pub endpoint: String,
    pub api_key: String,
    pub client: Client,
}

#[async_trait]
impl Adapter for WhatsAppAdapter {
    fn name(&self) -> &str {
        "whatsapp"
    }

    async fn send(&self, chat_id: &str, message: OutgoingMessage) -> anyhow::Result<DeliveryReceipt> {
        let text = match message {
            OutgoingMessage::Text { text } => text,
            OutgoingMessage::Media { caption, .. } => caption.unwrap_or_default(),
            OutgoingMessage::Event { .. } => String::new(),
        };
        let body = serde_json::json!({"to": chat_id, "text": text});
        let resp = self.client.post(&self.endpoint).bearer_auth(&self.api_key).json(&body).send().await?;
        if resp.status().is_success() {
            Ok(DeliveryReceipt { id: chat_id.into(), channel: "whatsapp".into(), ok: true, error: None })
        } else {
            Ok(DeliveryReceipt { id: chat_id.into(), channel: "whatsapp".into(), ok: false, error: Some(resp.status().to_string()) })
        }
    }

    async fn poll_updates(&self) -> anyhow::Result<Vec<ChannelEnvelope>> {
        if self.endpoint.is_empty() || self.session_name.is_empty() {
            return Ok(vec![]);
        }
        let url = format!("{}/{}", self.endpoint.trim_end_matches('/'), self.session_name);
        let resp = self.client.get(url).bearer_auth(&self.api_key).send().await?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let val: serde_json::Value = resp.json().await?;
        let mut out = Vec::new();
        if let Some(items) = val.as_array().or_else(|| val.get("messages").and_then(|x| x.as_array())) {
            for item in items {
                let text = item.get("text").and_then(|x| x.as_str()).unwrap_or_default().to_string();
                let from = item.get("from").and_then(|x| x.as_str()).or_else(|| item.get("author").and_then(|x| x.as_str())).map(|s| s.to_string());
                out.push(ChannelEnvelope {
                    channel: "whatsapp".into(),
                    chat_id: from.clone().unwrap_or_default(),
                    from,
                    message: IncomingMessage::Text { text },
                    raw: Some(item.clone()),
                });
            }
        }
        Ok(out)
    }
}
