use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Conversation {
    pub id: String,
    pub channel: String,
    pub chat_id: String,
    pub messages: Vec<Message>,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Clone)]
pub struct ConversationState {
    conversations: Arc<RwLock<HashMap<String, Conversation>>>,
    user_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ConversationState {
    pub fn new() -> Self {
        Self {
            conversations: Arc::new(RwLock::new(HashMap::new())),
            user_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(&self, channel: &str, chat_id: &str) -> Conversation {
        let id = Uuid::new_v4().to_string();
        let conv = Conversation {
            id: id.clone(),
            channel: channel.to_string(),
            chat_id: chat_id.to_string(),
            messages: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
        };
        self.conversations.write().await.insert(id.clone(), conv.clone());
        self.user_index.write().await
            .entry(format!("{}:{}", channel, chat_id))
            .or_default()
            .push(id.clone());
        conv
    }

    pub async fn get(&self, id: &str) -> Option<Conversation> {
        self.conversations.read().await.get(id).cloned()
    }

    pub async fn add_message(&self, conv_id: &str, role: &str, content: &str) -> anyhow::Result<()> {
        let mut convs = self.conversations.write().await;
        if let Some(conv) = convs.get_mut(conv_id) {
            conv.messages.push(Message {
                id: Uuid::new_v4().to_string(),
                role: role.to_string(),
                content: content.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            });
        }
        Ok(())
    }

    pub async fn find_by_user(&self, channel: &str, chat_id: &str) -> Option<Conversation> {
        let key = format!("{}:{}", channel, chat_id);
        let index = self.user_index.read().await;
        if let Some(ids) = index.get(&key) {
            if let Some(id) = ids.last() {
                return self.conversations.read().await.get(id).cloned();
            }
        }
        None
    }
}
