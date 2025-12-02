use lapin::{options::*,
    types::FieldTable, BasicProperties, Connection, ConnectionProperties, BasicProperties};
use serde_json::json;
use tracing::{info, error};
use crate::models::Event;
use dotenvy::dotenv;
use std::env;
use redis::AsyncCommands;
use std::sync::Arc;


pub async fn publish_example_event(ev: Event) -> anyhow::Result<()> {
    dotenv().ok();
    let amqp_addr = env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into());

    let conn = Connection::connect(&amqp_addr, ConnectionProperties::default().with_default_executor(8)).await?;
    let channel = conn.create_channel().await?;
    // Use a topic exchange so services/consumers can select
    let exchange_name = "events_topic";
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
