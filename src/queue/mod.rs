use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    pub backend: String,
    pub path: Option<PathBuf>,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self { backend: "memory".into(), path: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMessage {
    pub id: String,
    pub channel: String,
    pub chat_id: String,
    pub message: serde_json::Value,
}

#[async_trait::async_trait]
pub trait QueueBackend: Send + Sync {
    async fn enqueue(&self, msg: QueueMessage) -> anyhow::Result<String>;
    async fn dequeue(&self) -> anyhow::Result<Option<QueueMessage>>;
    async fn ack(&self, id: &str) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct MemoryQueue {
    #[allow(dead_code)]
    tx: mpsc::Sender<QueueMessage>,
}

impl MemoryQueue {
    pub fn new(capacity: usize) -> (Self, mpsc::Receiver<QueueMessage>) {
        let (tx, rx) = mpsc::channel(capacity);
        (Self { tx }, rx)
    }
}

#[async_trait::async_trait]
impl QueueBackend for MemoryQueue {
    async fn enqueue(&self, msg: QueueMessage) -> anyhow::Result<String> {
        self.tx.send(msg.clone()).await?;
        Ok(msg.id)
    }
    async fn dequeue(&self) -> anyhow::Result<Option<QueueMessage>> {
        Ok(None)
    }
    async fn ack(&self, _id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

pub mod persistent;


