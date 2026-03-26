use redis::{AsyncCommands, Client, RedisError};
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct RedisPublisher {
    client: Option<Client>,
    enabled: bool,
}

impl RedisPublisher {
    /// Creates a new instance with the provided dependencies.
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client: Some(client),
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
            let Some(client) = &self.client else {
                return Err(redis::RedisError::from((
                    redis::ErrorKind::ClientError,
                    "Redis client not configured",
                )));
            };
            match client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    let result: Result<(), RedisError> =
                        conn.publish(channel, payload.clone()).await;
                    if result.is_ok() {
                        return Ok(());
                    }
                    if attempts >= 3 {
                        return result;
                    }
                }
                Err(e) => {
                    if attempts >= 3 {
                        return Err(e);
                    }
                }
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Creates a disabled publisher that drops publish calls.
    pub fn new_noop() -> Self {
        Self {
            client: None,
            enabled: false,
        }
    }
}
