use crate::models::ProductEvent;
use dotenvy::dotenv;
use lapin::{BasicProperties, Connection, ConnectionProperties, options::*, types::FieldTable};
use std::env;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("json serialisation error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("rabbitMQ error: {0}")]
    Rabbit(#[from] lapin::Error),
}

/// Publishes a ProductEvent to the "analytics_events_topic" RabbitMQ exchange using the event's `event_type` as the routing key.
///
/// The function loads AMQP configuration from the `AMQP_ADDR` environment variable (defaults to
/// `amqps://guest:guest@127.0.0.1:5671/%2f`), establishes a connection and channel, enables publisher
/// confirms, declares the `analytics_events_topic` exchange (topic), serializes the event to JSON,
/// publishes the message with delivery mode 2 (persistent), and awaits broker confirmation.
///
/// # Parameters
/// - `ev`: The ProductEvent to publish; its `event_type` field is used as the routing key.
///
/// # Errors
/// Returns `PublishError::Json` if serialization to JSON fails, or `PublishError::Rabbit` for connection,
/// channel, exchange declaration, publish, or confirm-related failures.
///
/// # Examples
///
/// ```
/// # use crate::models::ProductEvent;
/// # use crate::publisher::publish_example_event;
/// # // The example below assumes a running RabbitMQ instance and appropriate crate layout.
/// # tokio_test::block_on(async {
/// let ev = ProductEvent {
///     event_type: "product.created".into(),
///     ..Default::default()
/// };
/// let result = publish_example_event(&ev).await;
/// assert!(result.is_ok());
/// # });
/// ```
pub async fn publish_example_event(ev: &ProductEvent) -> Result<(), PublishError> {
    println!("[DEBUG] Starting publish_example_event_to_rabbitMQ");

    dotenv().ok();
    // println!("[DEBUG] Loaded .env");

    let amqp_addr =
        env::var("AMQP_ADDR").unwrap_or_else(|_| "amqps://guest:guest@127.0.0.1:5671/%2f".into());
    // println!("[DEBUG] AMQP_ADDR = {:?}", amqp_addr);

    // Connect to RabbitMQ
    println!("[DEBUG] Attempting connection to RabbitMQ...");
    let conn = match Connection::connect(&amqp_addr, ConnectionProperties::default()).await {
        Ok(c) => {
            // println!("[DEBUG] Connection established successfully");
            c
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to connect to RabbitMQ: {:?}", e);
            return Err(e.into());
        }
    };

    // Create a channel
    println!("[DEBUG] Creating channel...");
    let channel = match conn.create_channel().await {
        Ok(ch) => {
            // println!("[DEBUG] Channel created successfully");
            ch
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to create channel: {:?}", e);
            return Err(e.into());
        }
    };

    // Enable publisher confirms
    // println!("[DEBUG] Selecting confirm mode...");
    if let Err(e) = channel
        .confirm_select(ConfirmSelectOptions::default())
        .await
    {
        eprintln!("[ERROR] Failed to select confirm mode: {:?}", e);
        return Err(e.into());
    }
    // println!("[DEBUG] Confirm mode enabled");

    // Declare exchange
    let exchange_name = "analytics_events_topic";
    // println!("[DEBUG] Declaring exchange '{}'", exchange_name);
    if let Err(e) = channel
        .exchange_declare(
            exchange_name,
            lapin::ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
    {
        eprintln!("[ERROR] Failed to declare exchange: {:?}", e);
        return Err(e.into());
    }
    // println!("[DEBUG] Exchange declared successfully");

    // Prepare payload
    let routing_key = ev.event_type.clone();
    // println!("[DEBUG] Preparing payload for routing key '{}'", routing_key);

    let payload = match serde_json::to_vec(&ev) {
        Ok(p) => {
            // println!("[DEBUG] Payload serialized successfully ({} bytes)", p.len());
            p
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to serialize payload: {:?}", e);
            return Err(e.into());
        }
    };

    // Publish message
    // println!("[DEBUG] Publishing message...");
    match channel
        .basic_publish(
            exchange_name,
            &routing_key,
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default().with_delivery_mode(2),
        )
        .await
    {
        Ok(confirm) => {
            // println!("[DEBUG] Publish sent, awaiting confirmation...");
            if let Err(e) = confirm.await {
                eprintln!("[ERROR] Publish confirmation failed: {:?}", e);
                return Err(e.into());
            }
            // println!("[DEBUG] Message published and confirmed successfully");
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to publish message: {:?}", e);
            return Err(e.into());
        }
    }

    info!("Published event {:?}", ev.event_type);
    println!("[DEBUG] Finished publish_example_event");

    Ok(())
}
