use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::queue::QueueMessage;

pub struct PersistentQueue {
    conn: Arc<Mutex<Connection>>,
}

impl PersistentQueue {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("CREATE TABLE IF NOT EXISTS queue (id TEXT PRIMARY KEY, channel TEXT, chat_id TEXT, message TEXT)")?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }
}

#[async_trait::async_trait]
impl crate::queue::QueueBackend for PersistentQueue {
    async fn enqueue(&self, msg: QueueMessage) -> anyhow::Result<String> {
        let c = self.conn.lock().await;
        c.execute(
            "INSERT OR REPLACE INTO queue (id, channel, chat_id, message) VALUES (?1, ?2, ?3, ?4)",
            params![msg.id, msg.channel, msg.chat_id, msg.message.to_string()]
        )?;
        Ok(msg.id)
    }
    async fn dequeue(&self) -> anyhow::Result<Option<QueueMessage>> {
        let c = self.conn.lock().await;
        let mut stmt = c.prepare("SELECT id, channel, chat_id, message FROM queue LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(QueueMessage {
                id: row.get(0)?,
                channel: row.get(1)?,
                chat_id: row.get(2)?,
                message: serde_json::from_str(&row.get::<_, String>(3)?)?,
            }))
        } else { Ok(None) }
    }
    async fn ack(&self, id: &str) -> anyhow::Result<()> {
        let c = self.conn.lock().await;
        c.execute("DELETE FROM queue WHERE id = ?1", params![id])?;
        Ok(())
    }
}
