use lapin::{Connection, ConnectionProperties, BasicProperties, Error};
use lapin::types::{FieldTable, AMQPValue};
use lapin::options::{ ExchangeDeclareOptions, QueueDeclareOptions, QueueBindOptions, BasicRejectOptions, BasicAckOptions, BasicPublishOptions, BasicConsumeOptions};
use sqlx::PgPool;
use tracing::{info, error};
use crate::models::Event;
use dotenvy::dotenv;
use std::env;
use actix_web::web;
use uuid::Uuid;
use redis::AsyncCommands;
use tracing::warn;
use serde_json::Value;
use futures_util::StreamExt;
use chrono::{DateTime, Utc};

pub struct RabbitConsumer {
    pub pool: PgPool
}

impl RabbitConsumer {
    pub fn new(
        pool: &PgPool
    ) -> Self {
        Self {
            pool: pool.clone()
        }
    }

    pub async fn run(
        &self,
        pool: &PgPool,
        redis_client: &web::Data<redis::Client>
    ) -> Result<(), Error> {
        dotenv().ok();
        let amqp_addr = env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into());

        // RabbitMQ
        let rabbit_conn = Connection::connect(
            &amqp_addr,
            ConnectionProperties::default()).await?;
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
            AMQPValue::LongString("".into()),
        );
        args.insert(
            "x-dead-letter-routing-key".into(),
            AMQPValue::LongString("analytics_dlq".into()),
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
            "#", // this makes it behave like fanout exchange where we wont have bothered putting anything here, i.e. "" in fanout
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

        while let Some(delivery_result) = consumer.next().await {
            let delivery = match delivery_result {
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
                .as_ref()
                .and_then(|h| h.inner().get("x-retries"))
                .and_then(|v| match v {
                    AMQPValue::LongLongInt(i) => Some(*i as i64),
                    AMQPValue::LongInt(i) => Some(*i as i64),
                    AMQPValue::ShortInt(i) => Some(*i as i64),
                    AMQPValue::ShortShortInt(i) => Some(*i as i64),
                    _ => None
                })
                .unwrap_or(3);

            let analytics_event: Event = match serde_json::from_slice(&delivery.data) {
                Ok(ev)  => match Event::new(ev) {
                    Ok(ev) => ev,
                    Err(err) => {
                        warn!("Failed converting Event -> AnalyticsEvent: {}", err);
                        // You can ack or reject depending on importance
                        channel.basic_ack(delivery.delivery_tag, BasicAckOptions::default()).await?;
                        continue;
                    }
                },
                Err(e) => {
                    error!("Bad JSON in message: {:?}", e);
                    // These are useless; send to DLQ
                    channel.basic_reject(
                        delivery.delivery_tag,
                        BasicRejectOptions { requeue: false }
                    ).await?;
                    continue;
                }
            };

            // process event 
            let db_res = insert_event(&pool, &analytics_event).await;
            let redis_res = update_redis(&analytics_event.data, &redis_client).await;

            if db_res.is_ok() && redis_res.is_ok() {
                // everything ok → normal ack
                channel.basic_ack(delivery.delivery_tag,
                BasicAckOptions::default()).await?;
                info!("Processed event (ok): {}", &analytics_event.event_type);
                continue;
            }

            // Failure path
            error!("Processing failed (db or redis): retry = {}", retries);

            if retries >= 3 {
                // Send to DLQ — final failure
                channel.basic_reject(
                    delivery.delivery_tag,
                    BasicRejectOptions { requeue: false }
                ).await?;
                error!("Message sent to DLQ");
                continue;
            }

            // Requeue with incremented retry header
            let mut headers = FieldTable::default();
            headers.insert(
                "x-retries".into(),
                AMQPValue::LongLongInt((retries + 1) as i64));

            channel.basic_publish(
                "exchange_name",
                "analytics_queue",
                BasicPublishOptions::default(),
                &delivery.data,
                BasicProperties::default().with_headers(headers),
            ).await?;

            // Acknowledge original delivery (we already requeued a new one)
            channel.basic_ack(
                delivery.delivery_tag,
                BasicAckOptions::default()
            ).await?;
        }

    Ok(())
    }
}

async fn insert_event(
    pool: &PgPool,
    event: &Event
) -> Result<(), sqlx::Error> {
    let id_key = match &event.event_type {
        t if t.starts_with("order.") => "order_id",
        t if t.starts_with("product.") => "product_id",
        t if t.starts_with("user.") => "user_id",
        _ => "random",
    };

    let id = match event.data.get(id_key).and_then(|v| v.as_str()) {
        Some(s) => Uuid::parse_str(s).unwrap_or_else(|_| Uuid::new_v4()),
        None => Uuid::new_v4()
    };

    let timestamp: DateTime<Utc> = event.data
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| Utc::now());

    sqlx::query(
        r#"
            INSERT INTO analytics.events (id, event_type, event_timestamp, data)
            VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(id)
    .bind(&event.event_type)
    .bind(timestamp)
    .bind(&event.data)
    .execute(pool)
    .await?;

    Ok(())
}


async fn update_redis(
        event_data: &Value,
        redis_client: &web::Data<redis::Client>
    ) -> redis::RedisResult<i64> {
        let mut redis_conn = redis_client.get_async_connection().await?;
        let result = match event_data.get("event_type").and_then(|v| v.as_str()) {
            Some("product.viewed") => {
                if let Some(product_id) = event_data.get("product_id").and_then(|v| v.as_str()) {
                    let key = format!("product_view_count:{}", product_id);
                    redis_conn.incr(key, 1).await?
                } else { 0 }
            }
            Some("order.created") => {
                if let Some(order_id) = event_data.get("order_id").and_then(|w| w.as_str()) {
                    let key = format!("orders_placed_count:{}", order_id);
                    redis_conn.incr(key, 1).await?
                } else { 0 }
            }
            Some("user.created") => {
                if let Some(user_id) = event_data.get("user_id").and_then(|w| w.as_str()) {
                    let key = format!("users_created_count:{}", user_id);
                    redis_conn.incr(key, 1).await?
                } else { 0 }
            }
            _ => { 0 }
        };
    Ok(result)
}
