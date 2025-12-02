use lapin::{options::*,
    types::FieldTable, Connection, ConnectionProperties, Consumer, BasicProperties};
use sqlx::PgPool;
use tracing::{info, error};
use crate::models::Event;
use serde_json::Value;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use uuid::Uuid;
use redis::{AsyncCommands, Client, aio::Connection as Conn};

pub struct Consumer {
    pub pool: PgPool
}

impl Consumer {
    pub async fn run(
        &self,
        pool: &PgPool
    ) -> Result<()> {
        dotenv().ok();
        let amqp_addr = env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into());
        let pg_url = env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/analytics".into());
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".into());

        // Redis client
        let client = redis::Client::open(redis_url)?;
        let mut redis_conn = client.get_async_connection().await?;

        // RabbitMQ
        let rabbit_conn = Connection::connect(&amqp_addr, ConnectionProperties::default()).await?;
        let channel = rabbit_conn.create_channel().await?;
        let exchange_name = "analytics_events_topic";

        channel.exchange_declare(
            exchange_name,
            lapin::ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default()
        ).await?;

        // 1. Declare the DLQ
        channel.queue_declare(
            "analytics_dlq",
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        ).await?;

        // 2. Declare MAIN queue with DLQ routing
        let mut args = FieldTable::default();
        args.insert(
            "x-dead-letter-exchange".into(),
            FieldValue::LongString("".into()),
        );
        args.insert(
            "x-dead-letter-routing-key".into(),
            FieldValue::LongString("analytics_dlq".into()),
        );

        // Create a queue for analytics consumers; durable so we don't lose messages
        channel.queue_declare(
            "analytics_queue",
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            args,
        ).await?;

        // Bind to topic exchange
        channel.queue_bind(
            "analytics_queue",
            exchange_name,
            "#",
            QueueBindOptions::default(),
            FieldTable::default(),
        ).await?;


        let mut consumer = channel.basic_consume(
            "analytics_queue",
            "analytics_consumer_tag",
            BasicConsumeOptions::default(),
            FieldTable::default()
        ).await?;

        info!("Analytics worker started, waiting for messages...");

        while let Some(delivery) = consumer.next().await {
            let (channel, delivery) = match delivery_result {
                Ok(v) => v,
                Err(e) => {
                    error!("Consumer error: {:?}", e);
                    continue;
                }
            };
            // Extract retries from headers
            let retries = delivery
                .properties
                .headers()
                .and_then(|h| h.inner().get("x-retries"))
                .and_then(|v| v.as_i64())
                .unwrap_or(3);
        
            let event: Event = match serde_json::from_slice(&delivery.data) {
                Ok(ev) => ev,
                Err(e) => {
                    error!("Bad JSON in message: {:?}", e);
                    // These are useless; send to DLQ
                    channel.basic_reject(
                        delivery.delivery_tag(),
                        BasicRejectOptions { requeue: false }
                    ).await?;
                    continue;
                }
            };

            // process event 
            let db_res = insert_event(&pool, &event).await;
            let redis_res = update_redis(&event.data, redis_conn).await;

            if db_res.is_ok() && redis_res.is_ok() {
                // everything ok → normal ack
                channel.basic_ack(delivery.delivery_tag(), BasicAckOptions::default()).await?;
                info!("Processed event (ok): {}", event.event_type);
                continue;
            }

            // Failure path
            error!("Processing failed (db or redis): retry = {}", retries);

            if retries >= 3 {
                // Send to DLQ — final failure
                channel.basic_reject(
                    delivery.delivery_tag(),
                    BasicRejectOptions { requeue: false }
                ).await?;
                error!("Message sent to DLQ");
                continue;
            }

            // Requeue with incremented retry header
            let mut headers = FieldTable::default();
            headers.insert("x-retries".into(), FieldValue::LongLong((retries + 1) as i64));

            channel.basic_publish(
                "exchange_name",
                "analytics_queue",
                BasicPublishOptions::default(),
                &delivery.data,
                BasicProperties::default().with_headers(headers),
            ).await?;

            // Acknowledge original delivery (we already requeued a new one)
            channel.basic_ack(
                delivery.delivery_tag(),
                BasicAckOptions::default()
            ).await?;
        }

    Ok(())
}

async fn insert_event(
    pool: &PgPool,
    event: &Event
) -> anyhow::Result<()> {
    let id_key = match event.event_type.as_str() {
        t if t.starts_with("order.") => "order_id",
        t if t.starts_with("product.") => "product_id",
        t if t.starts_with("user.") => "user_id",
        _ => "random",
    };

    let id = if id_key == "random" {
        Uuid::new_v4().to_string()
    } else {
        event.data
            .get(id_key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string())
    };

    sqlx::query_as::<_, Event>(
        r#"
            INSERT INTO analytics.events (id, event_type, event_timestamp, data)
            VALUES ($1, $2, $3, $4)
        "#
    );
    .bind(id)
    .bind(&event.event_type)
    .bind(event.event_timestamp)
    .bind(&event.data)
    .execute(pool)
    .await?;

    Ok(())
}


async fn update_redis(
    &self,
    event: &Value,
    redis_conn: Conn
) -> redis::RedisResult<()> {
        match event.get("event_type").and_then(|v| v.as_str()) {
            Some("product.viewed") => {
                if let Some(product_id) = event.get("product_id").and_then(|v| v.as_str()) {
                    let key = format!("product_view_count:{}", product_id);
                    let _ = redis_conn.incr(key, 1).await?;
                }
            }
            Some("order.created") => {
                if let Some(order_id) = event.get("order_id").and_then(|w| w.as_str()) {
                    let key = format!("orders_placed_count:{}", order_id);
                    let _ = redis_conn.incr(key, 1).await?;
                }
            }
            Some("user.created") => {
                if let Some(user_id) = event.get("user_id").and_then(|w| w.as_str()) {
                    let key = format!("users_created_count:{}", user_id);
                    let _ = redis_conn.incr(key, 1).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    }
}

