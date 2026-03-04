use redis::{AsyncCommands, Client, RedisError};
use std::env;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct RedisPublisher {
    client: Client,
    enabled: bool,
}

impl RedisPublisher {
    /// Create a new `RedisPublisher` connected to the given Redis URL.
    ///
    /// The returned publisher is enabled and ready to publish messages.
    ///
    /// # Parameters
    ///
    /// - `redis_url`: Redis connection URL (e.g., `"redis://127.0.0.1/"`).
    ///
    /// # Returns
    ///
    /// `Ok(RedisPublisher)` with an initialized client and `enabled` set to `true` on success, or a `RedisError` if the client could not be created.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn run() -> Result<(), redis::RedisError> {
    /// let pubr = redis_backend::publisher::RedisPublisher::new("redis://127.0.0.1/").await?;
    /// assert!(pubr.enabled);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            enabled: true,
        })
    }

    /// Publish a message to a Redis channel, retrying the operation up to five times on failure.
    ///
    /// Serializes `message` to JSON and attempts to publish the serialized payload to `channel`.
    /// If this publisher is disabled, the call returns successfully without sending anything.
    /// On serialization failure or when all retry attempts are exhausted, a `RedisError` is returned.
    ///
    /// # Parameters
    ///
    /// - `channel`: Target Redis channel name for the published message.
    /// - `message`: Payload to serialize to JSON and publish.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the message was published or publishing was skipped because the publisher is disabled; `Err(RedisError)` with the serialization error or the last Redis error after exhausting retries.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// # use logistics::publisher::RedisPublisher;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let publisher = RedisPublisher::new_noop();
    /// let msg = json!({ "event": "test", "value": 1 });
    /// let res = publisher.publish("events", &msg).await;
    /// assert!(res.is_ok());
    /// # }
    /// ```
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

    /// Creates a disabled RedisPublisher that ignores publish requests.
    ///
    /// The returned publisher is configured with a Redis client but has publishing
    /// disabled; calls to `publish` will be no-ops.
    ///
    /// # Examples
    ///
    /// ```
    /// let pub_noop = RedisPublisher::new_noop();
    /// // publishing will be dropped silently
    /// let _ = pub_noop.publish("channel", &"message");
    /// assert!(!pub_noop.enabled);
    /// ```
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
