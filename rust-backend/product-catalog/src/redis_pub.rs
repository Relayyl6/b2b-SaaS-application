use redis::{AsyncCommands, Client, RedisError};
use std::env;
use tokio::sync::mpsc::{self, Sender};
use tokio::time::{Duration, sleep};

#[derive(Clone)]
pub struct RedisPublisher {
    client: Option<Client>,
    enabled: bool,
    redis_publish_sender: Option<Sender<(String, String)>>,
}

impl RedisPublisher {
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        let (tx, mut rx) = mpsc::channel::<(String, String)>(256);
        let worker_client = client.clone();

        tokio::spawn(async move {
            while let Some((channel, payload)) = rx.recv().await {
                let mut attempts = 0;
                loop {
                    attempts += 1;
                    match worker_client.get_multiplexed_async_connection().await {
                        Ok(mut conn) => {
                            let result = conn.publish::<_, _, ()>(&channel, payload.clone()).await;
                            if result.is_ok() || attempts >= 3 {
                                if let Err(e) = result {
                                    eprintln!(
                                        "❌ Redis publish worker failed for channel '{}' after {} attempts: {:?}",
                                        channel, attempts, e
                                    );
                                }
                                break;
                            }
                        }
                        Err(e) => {
                            if attempts >= 3 {
                                eprintln!(
                                    "⚠️ Redis publish worker reconnect failed for channel '{}' after {} attempts: {:?}",
                                    channel, attempts, e
                                );
                                break;
                            }
                        }
                    }

                    sleep(Duration::from_millis(100)).await;
                }
            }
        });

        Ok(Self {
            client: Some(client),
            enabled: true,
            redis_publish_sender: Some(tx),
        })
    }

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

        let sender = self.redis_publish_sender.as_ref().ok_or_else(|| {
            RedisError::from((
                redis::ErrorKind::ClientError,
                "Redis publish queue not configured",
            ))
        })?;

        sender
            .try_send((channel.to_string(), payload))
            .map_err(|e| {
                RedisError::from((
                    redis::ErrorKind::BusyLoadingError,
                    "Redis publish queue is full or closed",
                    e.to_string(),
                ))
            })
    }

    pub fn new_noop() -> Self {
        Self {
            client: None,
            enabled: false,
            redis_publish_sender: None,
        }
    }

    pub fn client(&self) -> Option<&Client> {
        self.client.as_ref()
    }
}
