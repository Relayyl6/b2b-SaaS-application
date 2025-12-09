use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties, BasicProperties};
use tracing::info;
use crate::models::Event;
use dotenvy::dotenv;
use std::env;
use tracing::error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("json serialisation error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("rabbitMQ error: {0}")]
    Rabbit(#[from] lapin::Error),
}

pub async fn publish_example_event(
    ev: Event
) -> Result<(), PublishError> {
    dotenv().ok();
    let amqp_addr = env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into());

    let conn = Connection::connect(&amqp_addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    // Use a topic exchange so services/consumers can select
    let exchange_name = "analytics_events_topic";
    channel.exchange_declare(
        exchange_name,
        lapin::ExchangeKind::Topic,
        ExchangeDeclareOptions::default(),
        FieldTable::default()
    ).await?;

    let routing_key = ev.event_type.clone(); // e.g., "log.created"

    let payload = serde_json::to_vec(&ev)?;

    channel.basic_publish(
        exchange_name,
        &routing_key,
        BasicPublishOptions::default(),
        &payload,
        BasicProperties::default().with_delivery_mode(2) // persistent
    ).await?.await?; // wait for confirm

    info!("Published event {:?}", ev.event_type);
    Ok(())
}
