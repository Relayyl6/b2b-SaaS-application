use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties, BasicProperties};
use tracing::info;
use crate::models::ProductEvent;
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
    ev: &ProductEvent
) -> Result<(), PublishError> {
    println!("[DEBUG] Starting publish_example_event_to_rabbitMQ");

    dotenv().ok();
    // println!("[DEBUG] Loaded .env");

    let amqp_addr = env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqps://guest:guest@127.0.0.1:5671/%2f".into());
    // println!("[DEBUG] AMQP_ADDR = {:?}", amqp_addr);

    // Connect to RabbitMQ
    println!("[DEBUG] Attempting connection to RabbitMQ...");
    let conn = match Connection::connect(&amqp_addr, ConnectionProperties::default()).await {
        Ok(c) => {
            // println!("[DEBUG] Connection established successfully");
            c
        },
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
        },
        Err(e) => {
            eprintln!("[ERROR] Failed to create channel: {:?}", e);
            return Err(e.into());
        }
    };

    // Enable publisher confirms
    // println!("[DEBUG] Selecting confirm mode...");
    if let Err(e) = channel.confirm_select(ConfirmSelectOptions::default()).await {
        eprintln!("[ERROR] Failed to select confirm mode: {:?}", e);
        return Err(e.into());
    }
    // println!("[DEBUG] Confirm mode enabled");

    // Declare exchange
    let exchange_name = "analytics_events_topic";
    // println!("[DEBUG] Declaring exchange '{}'", exchange_name);
    if let Err(e) = channel.exchange_declare(
        exchange_name,
        lapin::ExchangeKind::Topic,
        ExchangeDeclareOptions::default(),
        FieldTable::default()
    ).await {
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
        },
        Err(e) => {
            eprintln!("[ERROR] Failed to serialize payload: {:?}", e);
            return Err(e.into());
        }
    };

    // Publish message
    // println!("[DEBUG] Publishing message...");
    match channel.basic_publish(
        exchange_name,
        &routing_key,
        BasicPublishOptions::default(),
        &payload,
        BasicProperties::default().with_delivery_mode(2)
    ).await {
        Ok(confirm) => {
            // println!("[DEBUG] Publish sent, awaiting confirmation...");
            if let Err(e) = confirm.await {
                eprintln!("[ERROR] Publish confirmation failed: {:?}", e);
                return Err(e.into());
            }
            // println!("[DEBUG] Message published and confirmed successfully");
        },
        Err(e) => {
            eprintln!("[ERROR] Failed to publish message: {:?}", e);
            return Err(e.into());
        }
    }

    info!("Published event {:?}", ev.event_type);
    println!("[DEBUG] Finished publish_example_event");

    Ok(())
}














