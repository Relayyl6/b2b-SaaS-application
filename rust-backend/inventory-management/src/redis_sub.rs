// src/redis_sub.rs
// Listens for product events on Redis and hands them off to local event handlers.

use sqlx::PgPool;

use std::env;
use redis::{Client, aio::Connection};
use crate::redis_sub::events::ProductEvent;
use serde_json;
use tokio;

mod events;
use events::{create_product_from_event, update_product_from_event, delete_product_from_event};

use futures_util::StreamExt;

#[allow(deprecated)]
pub async fn listen_to_redis_events(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set in environment")?;


    // Main loop: wait for messages and handle each one.
    loop {
        println!("🔄 Connecting to Redis...");
        let client = Client::open(redis_url.as_str())?;

        let conn: Connection = match client.get_async_connection().await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Failed to connect to Redis: {:?}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
            };
        let mut pubsub = conn.into_pubsub();

        if let Err(e) = pubsub.subscribe("product.events").await {
                eprintln!("❌ Failed to subscribe: {:?}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        println!("📡 Subscribed to product.events");


        let mut stream = pubsub.on_message();



        while let Some(msg) = stream.next().await {
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to get payload from message: {:?}", e);
                    // skip this message and continue listening
                    continue;
                }
            };


            let parsed: Result<ProductEvent, _> = serde_json::from_str(&payload);

            let event = match parsed {
                Ok(ev) => ev,
                Err(e) => {
                    eprintln!("Failed to parse ProductEvent JSON: {} -- payload: {}", e, payload);
                    continue;
                }
            };

            match event.event_type.as_str() {
                "product.created" => {
                    if let Err(e) = create_product_from_event(&pool, event.clone()).await {
                        eprintln!("Error handling product.created: {:?}", e);
                    }
                }
                "product.updated" => {
                    if let Err(e) = update_product_from_event(&pool, event.clone()).await {
                        eprintln!("Error handling product.updated: {:?}", e);
                    }
                }
                "product.deleted" => {
                    if let Err(e) = delete_product_from_event(&pool, event.clone()).await {
                        eprintln!("Error handling product.deleted: {:?}", e);
                    }
                }
                other => {
                    // Unknown event type — log and continue.
                    eprintln!("Received unexpected product event type: {}", other);
                }
            }
        }

        eprintln!("⚠️ Redis stream closed — reconnecting in 5 seconds...");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
