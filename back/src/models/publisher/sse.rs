use serde_json::json;
use tokio::sync::broadcast;

use super::types::PublishLog;

#[derive(Clone)]
pub struct SseBroadcaster {
    tx: broadcast::Sender<String>,
}

impl SseBroadcaster {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast(&self, log: &PublishLog) {
        let data = json!({
            "id": log.id,
            "publisher_id": log.publisher_id,
            "publisher_name": log.publisher_name,
            "publisher_type": log.publisher_type,
            "episode_title": log.episode_title,
            "status": log.status,
            "message": log.message,
            "created_at": log.created_at,
        });
        let _ = self.tx.send(data.to_string());
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

impl Default for SseBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}