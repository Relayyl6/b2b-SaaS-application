use crate::models::ProductEvent;
use dotenvy::dotenv;
use lapin::{
    BasicProperties, Connection, ConnectionProperties, options::*, publisher_confirm::Confirmation,
    types::FieldTable,
};
use std::env;
use thiserror::Error;
use tokio::time::{Duration, timeout};
use tracing::info;

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("json serialisation error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("rabbitMQ error: {0}")]
    Rabbit(#[from] lapin::Error),

    #[error("rabbitMQ timeout at step: {0}")]
    Timeout(&'static str),

    #[error("rabbitMQ publish was not acknowledged")]
    NotAcknowledged,
}

/// Publishes a product event message to RabbitMQ.
pub async fn publish_example_event(ev: &ProductEvent) -> Result<(), PublishError> {
    println!("[DEBUG] Starting publish_example_event_to_rabbitMQ");

    dotenv().ok();

    let amqp_addr =
        env::var("AMQP_ADDR").unwrap_or_else(|_| "amqps://guest:guest@127.0.0.1:5671/%2f".into());

    println!("[DEBUG] Attempting connection to RabbitMQ...");
    let conn = timeout(
        Duration::from_secs(2),
        Connection::connect(&amqp_addr, ConnectionProperties::default()),
    )
    .await
    .map_err(|_| PublishError::Timeout("connect"))??;

    println!("[DEBUG] Creating channel...");
    let channel = timeout(Duration::from_secs(2), conn.create_channel())
        .await
        .map_err(|_| PublishError::Timeout("create_channel"))??;

    timeout(
        Duration::from_secs(2),
        channel.confirm_select(ConfirmSelectOptions::default()),
    )
    .await
    .map_err(|_| PublishError::Timeout("confirm_select"))??;

    let exchange_name = "analytics_events_topic";
    timeout(
        Duration::from_secs(2),
        channel.exchange_declare(
            exchange_name,
            lapin::ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        ),
    )
    .await
    .map_err(|_| PublishError::Timeout("exchange_declare"))??;

    let routing_key = ev.event_type.clone();
    let payload = serde_json::to_vec(&ev)?;

    let confirm = timeout(
        Duration::from_secs(2),
        channel.basic_publish(
            exchange_name,
            &routing_key,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default().with_delivery_mode(2),
        ),
    )
    .await
    .map_err(|_| PublishError::Timeout("basic_publish"))??;

    match timeout(Duration::from_secs(2), confirm)
        .await
        .map_err(|_| PublishError::Timeout("publish_confirm"))??
    {
        Confirmation::Ack(_) => {}
        Confirmation::Nack(_) | Confirmation::NotRequested => {
            return Err(PublishError::NotAcknowledged);
        }
    }

    timeout(Duration::from_secs(2), channel.close(200, "Bye"))
        .await
        .map_err(|_| PublishError::Timeout("channel_close"))??;
    timeout(Duration::from_secs(2), conn.close(200, "Bye"))
        .await
        .map_err(|_| PublishError::Timeout("connection_close"))??;

    info!("Published event {:?}", ev.event_type);
    println!("[DEBUG] Finished publish_example_event");

    Ok(())
}
