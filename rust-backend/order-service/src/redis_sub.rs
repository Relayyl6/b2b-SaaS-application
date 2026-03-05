// src/redis_sub.rs
// Listens for inventory/logistics events on Redis and updates order state.

use crate::models::{OrderEvent, OrderStatus};
use futures_util::StreamExt;
use redis::{aio::Connection, Client};
use serde_json::Value;
use sqlx::PgPool;
use std::env;

mod events;
use events::{
    update_order_cancelled_event, update_order_confirmed_event, update_order_delivered_event,
    update_order_failed_event, update_order_shipped_event,
};

#[allow(deprecated)]
pub async fn listen_to_redis_events(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set in environment")?;

    loop {
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

        for channel in [
            "inventory.rejected",
            "inventory.reservation_expired",
            "inventory.reserved",
            "inventory.expired",
            "inventory.released",
            "inventory.finalized",
            "order.delivered",
            "logistics.shipment_updated",
            "logistics.shipment_cancelled",
        ] {
            if let Err(e) = pubsub.subscribe(channel).await {
                eprintln!("❌ Failed to subscribe to {}: {:?}", channel, e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        }

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let channel = msg.get_channel_name().to_string();
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to get payload from message: {:?}", e);
                    continue;
                }
            };

            if channel.starts_with("logistics.") {
                if let Err(e) = handle_logistics_event(&pool, &channel, &payload).await {
                    eprintln!("Failed handling logistics event: {e}");
                }
                continue;
            }

            let event: OrderEvent = match serde_json::from_str(&payload) {
                Ok(ev) => ev,
                Err(e) => {
                    eprintln!(
                        "Failed to parse OrderEvent JSON: {} -- payload: {}",
                        e, payload
                    );
                    continue;
                }
            };

            match channel.as_str() {
                "inventory.rejected" => {
                    let _ = update_order_failed_event(&pool, event.clone()).await;
                }
                "inventory.reservation_expired" | "inventory.expired" | "inventory.released" => {
                    let _ = update_order_cancelled_event(&pool, event.clone()).await;
                }
                "inventory.reserved" => {
                    let _ = update_order_confirmed_event(&pool, event.clone()).await;
                }
                "inventory.finalized" => {
                    let _ = update_order_shipped_event(&pool, event.clone()).await;
                }
                "order.delivered" => {
                    let _ = update_order_delivered_event(&pool, event.clone()).await;
                }
                other => eprintln!("Received unexpected event type: {}", other),
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn handle_logistics_event(
    pool: &PgPool,
    channel: &str,
    payload: &str,
) -> Result<(), sqlx::Error> {
    let value: Value = match serde_json::from_str(payload) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };

    let order_id = value
        .get("order_id")
        .and_then(|v| v.as_str())
        .and_then(|v| uuid::Uuid::parse_str(v).ok());

    let Some(order_id) = order_id else {
        return Ok(());
    };

    let status = match channel {
        "logistics.shipment_cancelled" => Some(OrderStatus::Cancelled),
        "logistics.shipment_updated" => match value.get("status").and_then(|v| v.as_str()) {
            Some("intransit") => Some(OrderStatus::Shipped),
            Some("delivered") => Some(OrderStatus::Delivered),
            Some("cancelled") => Some(OrderStatus::Cancelled),
            _ => None,
        },
        _ => None,
    };

    if let Some(status) = status {
        sqlx::query("UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(status)
            .bind(order_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}
