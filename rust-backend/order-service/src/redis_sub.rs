// src/redis_sub.rs
// Consumes workflow events from Redis Streams and updates order state.

use crate::models::{OrderEvent, OrderStatus};
use platform::metrics;
use platform::streams;
use serde_json::Value;
use sqlx::PgPool;
use std::env;

mod events;
use events::{
    update_order_cancelled_event, update_order_confirmed_event, update_order_delivered_event,
    update_order_failed_event, update_order_shipped_event,
};

const EVENTS: &[&str] = &[
    "inventory.rejected",
    "inventory.reservation_expired",
    "inventory.reserved",
    "inventory.expired",
    "inventory.released",
    "inventory.finalized",
    "order.delivered",
    "logistics.shipment_created",
    "logistics.shipment_updated",
    "logistics.shipment_cancelled",
];

pub async fn listen_to_redis_events(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set in environment")?;
    let consumer = env::var("CONSUMER_NAME").unwrap_or_else(|_| "order-service-1".to_string());

    streams::consume_json::<Value, _, _>(
        &redis_url,
        "order-service",
        &consumer,
        EVENTS,
        move |envelope| {
            let pool = pool.clone();
            async move {
                let event_type = envelope.event_type.clone();
                let result = handle_event(&pool, &event_type, envelope.payload).await;
                metrics::inc_event(
                    "order-service",
                    &envelope.stream,
                    &event_type,
                    if result.is_ok() { "ok" } else { "error" },
                );
                if let Err(e) = result {
                    eprintln!("order-service stream handler failed for {event_type}: {e:?}");
                }
            }
        },
    )
    .await
}

async fn handle_event(
    pool: &PgPool,
    event_type: &str,
    payload: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    if event_type.starts_with("logistics.") {
        return handle_logistics_event(pool, event_type, payload).await;
    }

    let event: OrderEvent = match serde_json::from_value(payload) {
        Ok(event) => event,
        Err(_) => return Ok(()),
    };

    match event_type {
        "inventory.rejected" => update_order_failed_event(pool, event).await,
        "inventory.reservation_expired" | "inventory.expired" | "inventory.released" => {
            update_order_cancelled_event(pool, event).await
        }
        "inventory.reserved" => update_order_confirmed_event(pool, event).await,
        "inventory.finalized" => update_order_shipped_event(pool, event).await,
        "order.delivered" => update_order_delivered_event(pool, event).await,
        _ => Ok(()),
    }
}

async fn handle_logistics_event(
    pool: &PgPool,
    event_type: &str,
    value: Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let order_id = value
        .get("order_id")
        .and_then(|v| v.as_str())
        .and_then(|v| uuid::Uuid::parse_str(v).ok());

    let Some(order_id) = order_id else {
        return Ok(());
    };

    let status = match event_type {
        "logistics.shipment_created" => Some(OrderStatus::Confirmed), // Optional mapping
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
