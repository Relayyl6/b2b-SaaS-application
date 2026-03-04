use redis::{AsyncCommands, Client, RedisError};
use serde_json;
use std::env;
use tokio::time::{Duration, sleep};

#[derive(Clone)]
pub struct RedisPublisher {
    client: Client,
    enabled: bool,
}

impl RedisPublisher {
    /// Creates a RedisPublisher configured with the given Redis URL.
    ///
    /// On success the returned publisher is enabled; on failure returns the `RedisError` produced
    /// when creating the Redis client.
    ///
    /// # Examples
    ///
    /// ```
    /// #[tokio::test]
    /// async fn create_publisher() {
    ///     let res = RedisPublisher::new("redis://127.0.0.1/").await;
    ///     assert!(res.is_ok());
    /// }
    /// ```
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            enabled: true,
        })
    }

    /// Publishes a JSON-serialized message to a Redis channel, retrying on transient failures.
    ///
    /// If the publisher is disabled, this method does nothing and returns `Ok(())`.
    /// On serialization failure the error is converted into a `RedisError` with kind `TypeError`.
    /// The method will attempt to publish up to 5 times, waiting 2 seconds between attempts; if publishing still fails after the final attempt the last error is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::RedisPublisher;
    /// #
    /// #[tokio::test]
    /// async fn publish_noop_example() {
    ///     let publisher = RedisPublisher::new_noop();
    ///     // new_noop() creates a disabled publisher; publish is a no-op and returns Ok(())
    ///     publisher.publish("my-channel", &"hello").await.unwrap();
    /// }
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

    /// Constructs a disabled `RedisPublisher` that ignores publish calls.
    ///
    /// The publisher is created with `enabled = false`. It attempts to construct a Redis client
    /// using the `REDIS_URL` environment variable and falls back to `redis://127.0.0.1/` if the
    /// variable is unset or the client cannot be created.
    ///
    /// # Examples
    ///
    /// ```
    /// let pub_noop = RedisPublisher::new_noop();
    /// // publish is a no-op and should return Ok(())
    /// let res = pub_noop.publish("channel", &serde_json::json!({"k":"v"}));
    /// assert!(res.is_ok());
    /// ```
    pub fn new_noop() -> Self {
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let client =
            Client::open(redis_url).unwrap_or_else(|_| Client::open("redis://127.0.0.1/").unwrap());
        Self {
            client,
            enabled: false,
        }
    }
}
