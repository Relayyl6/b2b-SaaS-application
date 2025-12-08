use redis::{AsyncCommands, Client, RedisError};
use serde_json;
use tokio::time::{sleep, Duration};
use std::env;

#[derive(Clone)]
pub struct RedisPublisher {
    client: Client,
    enabled: bool,
}

impl RedisPublisher {
    pub async fn new(
        redis_url: &str
    ) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            enabled: true,
        })
    }

    pub async fn publish<T: serde::Serialize>(&self, channel: &str, message: &T) -> Result<(), RedisError> {
        if !self.enabled {
            eprintln!("🟡 RedisPublisher disabled — skipping publish for channel '{}'", channel);
            return Ok(());
        }

        let payload = serde_json::to_string(message)
            .map_err(|e| redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Serialization failed",
                e.to_string(),
            )))?;


        let mut attempts = 0;

        loop {
            attempts += 1;
            match self.client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    let result = conn.publish(channel, payload.clone()).await;
                    if result.is_ok() {
                        return Ok(());
                    } else if attempts >= 5 {
                        eprintln!("❌ Redis publish failed after {} attempts", attempts);
                        return result;
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Redis reconnect failed (attempt {}): {:?}", attempts, e);
                    if attempts >= 5 {
                        return Err(e);
                    }
                }
            }

            sleep(Duration::from_secs(2)).await;
        }
    }

    pub fn new_noop() -> Self {
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let client = Client::open(redis_url).unwrap_or_else(|_| Client::open("redis://127.0.0.1/").unwrap());
        Self {
            client,
            enabled: false,
        }
    }
}
