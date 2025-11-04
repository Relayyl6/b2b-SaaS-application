use redis::{Client, RedisError};
use crate::models::ProductEvent;
use serde_json;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct RedisPublisher {
    conn: Arc<Mutex<redis::aio::MultiplexedConnection>>,
}

impl RedisPublisher {
    pub async fn new(url: &str) -> Result<Self, RedisError> {
        let client = Client::open(url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }

    pub async fn publish<T: serde::Serialize>(&self, channel: &str, event: &T) -> Result<(), RedisError> {
        let payload = serde_json::to_string(event).map_err(|e| RedisError::from((redis::ErrorKind::TypeError, e.to_string())))?;
        let mut lock = self.conn.lock().await;
        redis::aio::ConnectionLike::publish(&mut *lock, channel, payload).await
    }
}
