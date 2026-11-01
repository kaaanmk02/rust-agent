use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;

use crate::gateway::gateway_types::{Adapter, ChannelEnvelope, DeliveryReceipt, IncomingMessage, OutgoingMessage};

#[derive(Clone)]
pub struct TelegramAdapter {
    pub token: String,
    pub allowed_user_ids: Vec<i64>,
    pub client: Client,
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
        Ok(vec![])
    }
}

#[derive(Clone)]
pub struct DiscordAdapter {
    pub token: String,
    pub application_id: String,
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
        Ok(vec![])
    }
}

#[derive(Clone)]
pub struct SlackAdapter {
    pub bot_token: String,
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
        Ok(vec![])
    }
}

#[derive(Clone)]
pub struct WhatsAppAdapter {
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
        Ok(vec![])
    }
}
