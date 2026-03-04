use redis::{AsyncCommands, Client, RedisError};
use std::env;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct RedisPublisher {
    client: Client,
    enabled: bool,
}

impl RedisPublisher {
    /// Creates a new instance with the provided dependencies.
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            enabled: true,
        })
    }

    /// Publishes a serialized message to Redis with retry behavior.
    pub async fn publish<T: serde::Serialize>(
        &self,
        channel: &str,
        message: &T,
    ) -> Result<(), RedisError> {
        if !self.enabled {
            eprintln!(
                "🟡 RedisPublisher disabled — skipping publish for channel '{}'",
                channel
            );
            return Ok(());
        }

        let payload = serde_json::to_string(message).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Serialization failed",
                e.to_string(),
            ))
        })?;

        let mut attempts = 0;
        loop {
            attempts += 1;
            match self.client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    let result: Result<(), RedisError> =
                        conn.publish(channel, payload.clone()).await;
                    if result.is_ok() {
                        return Ok(());
                    }
                    if attempts >= 5 {
                        return result;
                    }
                }
                Err(e) => {
                    if attempts >= 5 {
                        return Err(e);
                    }
                }
            }

            sleep(Duration::from_secs(2)).await;
        }
    }

    /// Creates a disabled publisher that drops publish calls.
    pub fn new_noop() -> Self {
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let client = Client::open(redis_url)
            .unwrap_or_else(|_| Client::open("redis://127.0.0.1/").expect("fallback redis client"));

        Self {
            client,
            enabled: false,
        }
    }
}
