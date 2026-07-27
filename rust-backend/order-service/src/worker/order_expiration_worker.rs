use crate::models::OrderEvent;
use crate::redis_pub::RedisPublisher;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn start_order_expiration_worker(pool: PgPool, redis_pub: RedisPublisher) {
    tokio::spawn(async move {
        loop {
            // Find the next expiring pending order
            let next_expiry: Option<(Uuid, DateTime<Utc>)> = sqlx::query_as(
                "SELECT id, expires_at FROM orders WHERE status = 'pending' AND deleted_at IS NULL ORDER BY expires_at ASC LIMIT 1"
            )
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);

            if let Some((id, expires_at)) = next_expiry {
                let now = Utc::now();
                if expires_at <= now {
                    // Process this expired order
                    if let Err(e) = fail_expired_order(&pool, &redis_pub, id).await {
                        eprintln!("Order expiration worker error for order {}: {:?}", id, e);
                    }
                } else {
                    // Sleep exactly until the order expires
                    let duration = (expires_at - now).to_std().unwrap_or(std::time::Duration::from_secs(0));
                    tokio::time::sleep(duration).await;
                }
            } else {
                // No pending orders exist, sleep for 5 minutes before checking again
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
            }
        }
    });
}

async fn fail_expired_order(pool: &PgPool, redis_pub: &RedisPublisher, order_id: Uuid) -> Result<(), sqlx::Error> {
    let order_data: Option<(uuid::Uuid, uuid::Uuid, uuid::Uuid, chrono::DateTime<Utc>)> = sqlx::query_as(
        "SELECT product_id, user_id, supplier_id, expires_at FROM orders WHERE id = $1 AND status = 'pending'"
    )
    .bind(order_id)
    .fetch_optional(pool)
    .await?;

    if let Some((product_id, user_id, supplier_id, expires_at)) = order_data {
        // Log the audit transition
        let audit_id = Uuid::new_v4();
        let _ = sqlx::query(
            "INSERT INTO order_audit_logs (id, order_id, previous_status, new_status, changed_at) VALUES ($1, $2, $3, $4, NOW())"
        )
        .bind(audit_id)
        .bind(order_id)
        .bind("pending")
        .bind("failed")
        .execute(pool)
        .await?;

        sqlx::query("UPDATE orders SET status = 'failed' WHERE id = $1")
            .bind(order_id)
            .execute(pool)
            .await?;

        let fail_event = OrderEvent {
            tenant_id: Some(supplier_id),
            event_type: "order.failed".to_string(),
            order_id: Some(order_id),
            user_id: Some(user_id),
            product_id,
            supplier_id,
            timestamp: Utc::now(),
            expires_at,
            ..Default::default()
        };
        redis_pub.publish_async("order.failed", fail_event.clone());
        
        let release_cmd = OrderEvent { event_type: "inventory.release_command".to_string(), ..fail_event.clone() };
        redis_pub.publish_async("inventory.release_command", release_cmd);
        
        let refund_cmd = OrderEvent { event_type: "payment.refund_command".to_string(), ..fail_event.clone() };
        redis_pub.publish_async("payment.refund_command", refund_cmd);
    }

    Ok(())
}
