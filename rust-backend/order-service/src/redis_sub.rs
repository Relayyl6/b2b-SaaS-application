// src/redis_sub.rs
// Consumes workflow events from Redis Streams and updates order state.

use crate::models::{OrderEvent, OrderStatus};
use crate::redis_pub::RedisPublisher;
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

pub async fn listen_to_redis_events(pool: PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set in environment")?;
    let consumer = env::var("CONSUMER_NAME").unwrap_or_else(|_| "order-service-1".to_string());
    
    let redis_pub = RedisPublisher::new(&redis_url)
        .await
        .map_err(|e| format!("Failed to create RedisPublisher: {}", e))?;

    streams::consume_json::<Value, _, _>(
        &redis_url,
        "order-service",
        &consumer,
        EVENTS,
        move |envelope| {
            let pool = pool.clone();
            let redis_pub = redis_pub.clone();
            async move {
                let event_type = envelope.event_type.clone();

                let payload_tenant_id = envelope
                    .payload
                    .get("tenant_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| uuid::Uuid::parse_str(s).ok());
                let tenant_id = envelope.tenant_id.or(payload_tenant_id);

                if tenant_id.is_none() || tenant_id == Some(uuid::Uuid::nil()) {
                    tracing::warn!(%event_type, stream = %envelope.stream, "Missing tenant_id in stream event — skipping business logic");
                    metrics::inc_event("order-service", &envelope.stream, &event_type, "tenant_mismatch");
                    return Ok(());
                }

                if let (Some(env_tid), Some(pay_tid)) = (envelope.tenant_id, payload_tenant_id) {
                    if env_tid != pay_tid {
                        tracing::warn!(%event_type, ?env_tid, ?pay_tid, "Tenant ID mismatch between envelope and payload — skipping business logic");
                        metrics::inc_event("order-service", &envelope.stream, &event_type, "tenant_mismatch");
                        return Ok(());
                    }
                }

                let result = handle_event(&pool, &redis_pub, &event_type, envelope.payload).await;
                metrics::inc_event(
                    "order-service",
                    &envelope.stream,
                    &event_type,
                    if result.is_ok() { "ok" } else { "error" },
                );
                if let Err(e) = result {
                    eprintln!("order-service stream handler failed for {event_type}: {e:?}");
                    return Err(e);
                }
                Ok(())
            }
        },
    )
    .await
}

async fn handle_event(
    pool: &PgPool,
    redis_pub: &RedisPublisher,
    event_type: &str,
    payload: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if event_type.starts_with("logistics.") {
        return handle_logistics_event(pool, redis_pub, event_type, payload).await;
    }

    let event: OrderEvent = match serde_json::from_value(payload) {
        Ok(event) => event,
        Err(_) => return Ok(()),
    };

    match event_type {
        "inventory.rejected" => update_order_failed_event(pool, redis_pub, event).await,
        "inventory.reservation_expired" | "inventory.expired" | "inventory.released" => {
            update_order_cancelled_event(pool, redis_pub, event).await
        }
        "inventory.reserved" => update_order_confirmed_event(pool, redis_pub, event).await,
        "inventory.finalized" => update_order_shipped_event(pool, redis_pub, event).await,
        "order.delivered" => update_order_delivered_event(pool, redis_pub, event).await,
        _ => Ok(()),
    }
}

async fn handle_logistics_event(
    pool: &PgPool,
    redis_pub: &RedisPublisher,
    event_type: &str,
    value: Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        if let Ok(order) = crate::db::update_order_status_db(pool, order_id, status.clone(), None, None, None).await {
            // Replicate side effects
            if status == OrderStatus::Cancelled {
                let cancel_event = OrderEvent {
                    event_type: "order.cancelled".to_string(),
                    product_id: order.product_id,
                    supplier_id: order.supplier_id,
                    order_id: Some(order.id),
                    quantity: order.qty,
                    user_id: Some(order.user_id),
                    timestamp: order.order_timestamp,
                    ..Default::default()
                };
                redis_pub.publish_async("order.cancelled", cancel_event);
            } else if status == OrderStatus::Shipped {
                let shipped_event = OrderEvent {
                    event_type: "order.shipped".to_string(),
                    product_id: order.product_id,
                    supplier_id: order.supplier_id,
                    order_id: Some(order.id),
                    quantity: order.qty,
                    user_id: Some(order.user_id),
                    timestamp: order.order_timestamp,
                    ..Default::default()
                };
                redis_pub.publish_async("order.shipped", shipped_event);
            }
        }
    }

    Ok(())
}

