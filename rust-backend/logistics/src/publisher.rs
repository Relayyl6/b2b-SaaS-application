use redis::{AsyncCommands, Client, RedisError};
use std::env;
use tokio::time::{timeout, Duration};
use tracing::warn;

const REDIS_PUBLISH_TIMEOUT_MS: u64 = 300;

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

    /// Publishes a serialized message to Redis with a bounded timeout.
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
        }
    }

    async fn publish_payload(&self, channel: &str, payload: String) -> Result<(), RedisError> {
        if !self.enabled {
            return Ok(());
        }

        let publish_future = async {
            let mut conn = self.client.get_multiplexed_async_connection().await?;
            conn.publish::<_, _, ()>(channel, payload).await?;
            Ok::<(), RedisError>(())
        };

            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Publishes in a detached task so request handlers never block on Redis.
    pub fn publish_async<T>(&self, channel: &str, message: T)
    where
        T: serde::Serialize,
    {
        let payload = match serde_json::to_string(&message) {
            Ok(payload) => payload,
            Err(err) => {
                warn!(%channel, error = ?err, "failed to serialize redis payload");
                return;
            }
        };
        let this = self.clone();
        let channel = channel.to_string();
        tokio::spawn(async move {
            if let Err(err) = this.publish_payload(&channel, payload).await {
                warn!(%channel, error = ?err, "redis publish failed");
            }
        });
    }

    /// Creates a disabled publisher that drops publish calls.
    pub fn new_noop() -> Self {
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let client = Client::open(redis_url)
            .unwrap_or_else(|_| Client::open("redis://127.0.0.1/").expect("redis fallback"));

        Self {
            client: None,
            enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RedisPublisher;

    #[tokio::test]
    async fn noop_publisher_is_best_effort() {
        let pubw = RedisPublisher::new_noop();
        let result = pubw
            .publish("test.channel", &serde_json::json!({"ok": true}))
            .await;
        assert!(result.is_ok());
    }
}
