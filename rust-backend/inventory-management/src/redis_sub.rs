// src/redis_sub.rs
// Listens for product events on Redis and hands them off to local event handlers.

use sqlx::PgPool;

use std::env;
use redis::{Client, aio::Connection};
use crate::models::ProductEvent;
use serde_json;
use tokio;
use actix_web::web::Data;
// mod db;

use crate::redis_pub::RedisPublisher;

mod events;
use events::{create_product_from_event, update_product_from_event, delete_product_from_event, reserve_stock_from_order, release_stock_from_order, finalize_order_after_payment};
use crate::db::InventoryRepo;

use futures_util::StreamExt;

#[allow(deprecated)]
pub async fn listen_to_redis_events(
    pool: PgPool,
    repo: Data<InventoryRepo>,
    redis_pub: Data<RedisPublisher>
) -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set in environment")?;
    // let repo = web::Data::new(db::InventoryRepo::new(&pool));

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

        // Subscribe to all product channels in one go
        for channel in &["product.created", "product.updated", "product.deleted", "order.created", "order.cancelled", "order.failed", "payment.success"] {
            if let Err(e) = pubsub.subscribe(channel).await {
                eprintln!("❌ Failed to subscribe to {}: {:?}", channel, e);
                // wait before retrying subscription
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        }
        
        println!("📡 Subscribed to all product events");

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
                "order.created" => {
                    if let Err(e) = reserve_stock_from_order(&pool, redis_pub.clone(), event.clone()).await {
                        println!("Error handling order.created: {:?}", e);
                    }
                }
                "order.cancelled" => {
                    if let Err(e) = release_stock_from_order(&pool, redis_pub.clone(), event.clone()).await {
                        println!("Error handling order.cancelled: {:?}", e);
                    }
                }
                "order.failed" => {
                    if let Err(e) = release_stock_from_order(&pool, redis_pub.clone(), event.clone()).await {
                        println!("Error handling order.cancelled: {:?}", e);
                    }
                }
                "payment.success" => {
                    if let Err(e) = finalize_order_after_payment(&pool, redis_pub.clone(), repo.clone(), event.supplier_id, event.clone()).await {
                        println!("Error handling payment.success: {:?}", e);
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
