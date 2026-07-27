use crate::models::{AnalyticsEvent, Event};
use actix_web::web;
use platform::streams;
use serde_json::Value;
use sqlx::PgPool;
use std::env;
use tracing::{error, info, warn};

pub const EVENTS: &[&str] = &[
    "order.created",
    "order.confirmed",
    "order.cancelled",
    "order.shipped",
    "order.delivered",
    "product.created",
    "product.updated",
    "product.deleted",
    "product.viewed",
    "inventory.reserved",
    "inventory.updated",
    "inventory.lowstock",
    "inventory.released",
    "logistics.shipment_created",
    "logistics.shipment_updated",
    "logistics.shipment_cancelled",
    "payment.succeeded",
    "payment.failed",
    "payment.refunded",
    "user.created",
    "user.updated",
    "supplier.created",
    "supplier.status_updated",
    "supplier.updated",
    "notification.sent",
    "notification.failed",
];

pub async fn run_redis_consumer(
    pool: PgPool,
    redis_client: web::Data<redis::Client>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set in environment")?;
    let consumer = env::var("CONSUMER_NAME").unwrap_or_else(|_| "analytics-worker-1".to_string());

    info!("Starting analytics Redis stream consumer: {}", consumer);

    streams::consume_json::<Value, _, _>(
        &redis_url,
        "analytics",
        &consumer,
        EVENTS,
        move |envelope| {
            let pool = pool.clone();
            let redis_client = redis_client.clone();
            async move {
                let event_type = envelope.event_type.clone();
                let mut analytics_event: AnalyticsEvent = match serde_json::from_value(envelope.payload)
                {
                    Ok(ev) => ev,
                    Err(e) => {
                        warn!("Failed parsing AnalyticsEvent ({}): {}", event_type, e);
                        return Ok(());
                    }
                };

                let tenant_id = envelope.tenant_id.or(analytics_event.tenant_id);
                if tenant_id.is_none() || tenant_id == Some(uuid::Uuid::nil()) {
                    warn!(%event_type, stream = %envelope.stream, "Missing tenant_id in analytics stream event — skipping event ingestion");
                    return Ok(());
                }

                if analytics_event.tenant_id.is_none() {
                    analytics_event.tenant_id = tenant_id;
                }

                let event = match Event::new(analytics_event) {
                    Ok(ev) => ev,
                    Err(err) => {
                        warn!("Failed converting to Event: {}", err);
                        return Ok(());
                    }
                };

                let db_res = insert_event(&pool, &event).await;
                let redis_res = update_redis(&event.data, &redis_client).await;

                if let Err(e) = db_res {
                    error!("DB insertion failed: {}", e);
                }
                if let Err(e) = redis_res {
                    error!("Redis update failed: {}", e);
                }

                Ok(())
            }
        },
    )
    .await?;

    Ok(())
}

async fn insert_event(pool: &PgPool, event: &Event) -> Result<(), sqlx::Error> {
    let id_key = match &event.event_type {
        t if t.starts_with("order.") => "order_id",
        t if t.starts_with("product.") => "product_id",
        t if t.starts_with("user.") => "user_id",
        t if t.starts_with("logistics.") => "shipment_id",
        t if t.starts_with("inventory.") => "supplier_id",
        t if t.starts_with("payment.") => "payment_id",
        t if t.starts_with("supplier.") => "supplier_id",
        t if t.starts_with("notification.") => "notification_id",
        _ => "random",
    };

    let id = match event.data.get(id_key).and_then(|v| v.as_str()) {
        Some(s) => uuid::Uuid::parse_str(s).unwrap_or_else(|_| uuid::Uuid::new_v4()),
        None => uuid::Uuid::new_v4(),
    };

    let timestamp: chrono::DateTime<chrono::Utc> = event
        .data
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| chrono::Utc::now());

    sqlx::query(
        r#"
            INSERT INTO analytics.events (id, event_type, event_timestamp, data)
            VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(id)
    .bind(&event.event_type)
    .bind(timestamp)
    .bind(&event.data)
    .execute(pool)
    .await?;

    Ok(())
}

async fn update_redis(
    event_data: &Value,
    redis_client: &web::Data<redis::Client>,
) -> redis::RedisResult<i64> {
    let mut redis_conn = match redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(_) => return Ok(0),
    };
    use redis::AsyncCommands;
    let result = match event_data.get("event_type").and_then(|v| v.as_str()) {
        Some("product.viewed") => {
            if let Some(product_id) = event_data.get("product_id").and_then(|v| v.as_str()) {
                let key = format!("product_view_count:{}", product_id);
                redis_conn.incr(key, 1).await?
            } else {
                0
            }
        }
        Some("order.created") => {
            if let Some(order_id) = event_data.get("order_id").and_then(|w| w.as_str()) {
                let key = format!("orders_placed_count:{}", order_id);
                redis_conn.incr(key, 1).await?
            } else {
                0
            }
        }
        Some("user.created") => {
            if let Some(user_id) = event_data.get("user_id").and_then(|w| w.as_str()) {
                let key = format!("users_created_count:{}", user_id);
                redis_conn.incr(key, 1).await?
            } else {
                0
            }
        }
        _ => 0,
    };
    Ok(result)
}
