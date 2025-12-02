use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Connection,
    ConnectionProperties, Result,
};
use futures_util::stream::StreamExt; // For consuming messages
use crate::event::AnalyticsEvent;
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
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            enabled: true,
        })
    }

    pub async fn publish<T: serde::Serialize>(
        &self,
        channel: &str,
        message: &T
    ) -> Result<(), RedisError> {
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

                    let id_key = match payload.event_type.as_str() {
                        t if t.starts_with("order.") => "order_id",
                        t if t.starts_with("product.") => "product_id",
                        t if t.starts_with("user.") => "user_id",
                        _ => "random",
                    };

                    let id = if id_key == "random" {
                        uuid::Uuid::new_v4().to_string()
                    } else {
                        payload.data
                            .get(id_key)
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
                    };

                    let rabbit_result = publish_rabbit(AnalyticsEvent {
                            event_type: payload.event_type.clone(),
                            id: payload.log_id,
                            data: event.clone(),
                            event_timestamp: Utc::now()
                        });


                    if result.is_ok() {
                        return Ok(());
                    } else if rabbit_result.is_ok() {
                        return Ok(());
                    } else if attempts >= 3 {
                        eprintln!("❌ Redis publish failed after {} attempts", attempts);
                        return result;
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ RabbitMQ and redis reconnect failed (attempt {}): {:?}", attempts, e);
                    if attempts >= 3 {
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


pub async fn publish_rabbit(
    event: &AnalyticsEvent
) -> Result<()> {
    let amqp_addr = env::var("AMPQ_ADDR").unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".into())
    let conn = Connection::connect(
        amqp_addr,
        ConnectionProperties::default()
    ).await?;

    let channel = conn.create_channel().await?;

    channel
        .queue_declare(
            "analytics_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let event_clone = event.event_type.clone()
    let payload = serde_json::to_vec(event.clone())?;

    channel
        .basic_publish(
            "analytics_events_topic",
            event_clone,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default(),
        )
        .await?;

    println!("Published to RabbitMQ: {:?}", event);

    Ok(())
}




