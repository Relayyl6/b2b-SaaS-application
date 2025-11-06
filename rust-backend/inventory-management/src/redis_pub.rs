// src/redis_pub.rs
use redis::{AsyncCommands, Client};
use serde::Serialize;

#[derive(Clone)]
pub struct RedisPublisher {
    client: Client,
}

impl RedisPublisher {
    pub fn new(url: &str) -> Self {
        Self { client: Client::open(url).unwrap() }
    }

    pub async fn publish<T: Serialize>(
        &self,
        event: &T,
        channel: &str
    ) -> Result<(), redis::RedisError> {
        let payload = serde_json::to_string(event).unwrap();
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        conn.publish(channel, payload).await
    }
}