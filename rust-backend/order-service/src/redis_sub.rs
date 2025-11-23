// src/redis_sub.rs
// Listens for product events on Redis and hands them off to local event handlers.

use sqlx::PgPool;

use std::env;
use redis::{Client, aio::Connection};
use crate::models::OrderEvent;
use serde_json;
use tokio;

mod events;
use futures_util::StreamExt;

use events::{update_order_failed_event, update_order_confirmed_event, update_order_cancelled_event, update_order_shipped_event, update_order_delivered_event};

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

// Analogous events
// "inventory.expired", "order.failed", "inventory.rejected"
// "inventory.reserved", "order.confirmed"
// "inventory.released", "order.cancelled"
// "inventory.finalized", "order.shipped"
// "payment.success", "order.delivered"

// last one order.arrived



        // Subscribe to all product channels in one go
        for channel in &["inventory.rejected", "inventory.reservation_expired", "inventory.reserved", "inventory.expired", "inventory.released", "inventory.finalized", "order.delivered"] {
            if let Err(e) = pubsub.subscribe(channel).await {
                eprintln!("❌ Failed to subscribe to {}: {:?}", channel, e);
                // wait before retrying subscription
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        }
        
        println!("📡 Subscribed to all order events");

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


            let parsed: Result<OrderEvent, _> = serde_json::from_str(&payload);

            let event = match parsed {
                Ok(ev) => ev,
                Err(e) => {
                    eprintln!("Failed to parse OrderEvent JSON: {} -- payload: {}", e, payload);
                    continue;
                }
            };

            match event.event_type.as_str() {
                "inventory.rejected" => { //  "order.failed" 
                    if let Err(e) = update_order_failed_event(&pool, event.clone()).await {
                        eprintln!("Error handling product.created: {:?}", e);
                    }
                }
                "inventory.reservation_expired" => {  // "order.cancelled" 
                    if let Err(e) = update_order_cancelled_event(&pool, event.clone()).await {
                        eprintln!("Error handling product.created: {:?}", e);
                    }
                }
                "inventory.reserved" => { //  "order.confirmed" 
                    if let Err(e) = update_order_confirmed_event(&pool, event.clone()).await {
                        eprintln!("Error handling product.updated: {:?}", e);
                    }
                }
                "inventory.expired" => { //  "order.cancelled" 
                    if let Err(e) = update_order_cancelled_event(&pool, event.clone()).await {
                        eprintln!("Error handling product.deleted: {:?}", e);
                    }
                }
                "inventory.released" => { //  "order.cancelled" 
                    if let Err(e) = update_order_cancelled_event(&pool, event.clone()).await {
                        eprintln!("Error handling product.deleted: {:?}", e);
                    }
                }
                "inventory.finalized" => { //  "order.shipped" 
                    if let Err(e) = update_order_shipped_event(&pool, event.clone()).await {
                        println!("Error handling order.created: {:?}", e);
                    }
                }
                "order.delivered" => {
                    if let Err(e) = update_order_delivered_event(&pool, event.clone()).await {
                        println!("Error handling order.cancelled: {:?}", e);
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
