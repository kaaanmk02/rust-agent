use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;
#[derive(Clone)]
pub struct GracefulShutdown {
    pub tx: broadcast::Sender<()>,
}
impl GracefulShutdown {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1);
        Self { tx }
    }
    pub async fn wait(self: Arc<Self>) {
        let _ = self.tx.send(());
        info!("graceful shutdown triggered");
    }
}
