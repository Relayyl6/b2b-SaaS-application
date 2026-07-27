// src/redis_sub.rs
// Consumes product/order/payment workflow events from Redis Streams.

use actix_web::web::Data;
use platform::{metrics, streams};
use sqlx::PgPool;
use std::env;

use crate::db::InventoryRepo;
use crate::models::ProductEvent;
use crate::redis_pub::RedisPublisher;

mod events;
use events::{
    create_product_from_event, delete_product_from_event, finalize_order_after_payment,
    release_stock_from_order, reserve_stock_from_order, update_product_from_event,
};

const EVENTS: &[&str] = &[
    "product.created",
    "product.updated",
    "product.deleted",
    "order.created",
    "order.cancelled",
    "order.failed",
    "inventory.release_command",
    "payment.success",
    "payment.failed",
    "payment.cancelled",
];

pub async fn listen_to_redis_events(
    pool: PgPool,
    repo: Data<InventoryRepo>,
    redis_pub: Data<RedisPublisher>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set in environment")?;
    let consumer = env::var("CONSUMER_NAME").unwrap_or_else(|_| "inventory-1".to_string());

    streams::consume_json::<ProductEvent, _, _>(
        &redis_url,
        "inventory-management",
        &consumer,
        EVENTS,
        move |envelope| {
            let pool = pool.clone();
            let repo = repo.clone();
            let redis_pub = redis_pub.clone();
            async move {
                let event_type = envelope.event_type.clone();
                let event = envelope.payload;

                let tenant_id = envelope.tenant_id.or(event.tenant_id);
                if tenant_id.is_none() || tenant_id == Some(uuid::Uuid::nil()) {
                    tracing::warn!(%event_type, stream = %envelope.stream, "Missing tenant_id in stream event — skipping business logic");
                    metrics::inc_event("inventory-management", &envelope.stream, &event_type, "tenant_mismatch");
                    return Ok(());
                }

                if let (Some(env_tid), Some(pay_tid)) = (envelope.tenant_id, event.tenant_id) {
                    if env_tid != pay_tid {
                        tracing::warn!(%event_type, ?env_tid, ?pay_tid, "Tenant ID mismatch between stream envelope and payload — skipping business logic");
                        metrics::inc_event("inventory-management", &envelope.stream, &event_type, "tenant_mismatch");
                        return Ok(());
                    }
                }

                let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = match event_type.as_str() {
                    "product.created" => create_product_from_event(&pool, event).await,
                    "product.updated" => update_product_from_event(&pool, event).await,
                    "product.deleted" => delete_product_from_event(&pool, event).await,
                    "order.created" => reserve_stock_from_order(&pool, redis_pub, event).await,
                    "order.cancelled" | "order.failed" | "inventory.release_command" => {
                        release_stock_from_order(&pool, redis_pub, event).await
                    }
                    "payment.success" => {
                        finalize_order_after_payment(
                            &pool,
                            redis_pub,
                            repo,
                            event.supplier_id,
                            event,
                        )
                        .await
                    }
                    "payment.failed" | "payment.cancelled" => {
                        release_stock_from_order(&pool, redis_pub, event).await
                    }
                    _ => Ok(()),
                };

                metrics::inc_event(
                    "inventory-management",
                    &envelope.stream,
                    &event_type,
                    if result.is_ok() { "ok" } else { "error" },
                );
                if let Err(e) = result {
                    eprintln!("inventory stream handler failed for {event_type}: {e:?}");
                    return Err(e);
                }
                Ok(())
            }
        },
    )
    .await
}
