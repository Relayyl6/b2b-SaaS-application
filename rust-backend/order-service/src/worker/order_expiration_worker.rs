use chrono::Utc;
use sqlx::PgPool;
use tokio::time::{interval, Duration};
use serde_json::json;
use crate::redis_pub::RedisPublisher;

pub async fn start_order_expiration_worker(pool: PgPool, redis_pub: RedisPublisher) {
    tokio::spawn(async move {
        let mut timer = interval(Duration::from_secs(2 * 24 * 60 * 60));

        loop {
            timer.tick().await;

            match fail_expired_orders(&pool, &redis_pub).await {
                Ok(_) => println!("Order expiration check complete"),
                Err(e) => eprintln!("Order expiration worker error: {:?}", e),
            }
        }
    });
}

async fn fail_expired_orders(
    pool: &PgPool,
    redis_pub: &RedisPublisher
) -> Result<(), sqlx::Error> {
    let expired_orders = sqlx::query!(
        r#"
            SELECT id, product_id, user_id
            FROM orders
            WHERE status = 'pending'
            AND expires_at < NOW()
        "#
    )
    .fetch_all(pool)
    .await?;

    for order in expired_orders {
        sqlx::query!(
            r#"
                UPDATE orders
                SET status = 'failed'
                WHERE id = $1
            "#,
            order.id
        )
        .execute(pool)
        .await?;

        let event = json!({
            "event_type": "order.failed",
            "id": order.id,
            "user_id": order.user_id,
            "product_id": order.product_id,
            "new_status": "failed".to_string(),
            "timestamp": Utc::now().timestamp_millis(),
        });

        if let Err(e) = redis_pub.publish("order.failed", &event).await {
            eprintln!("Redis publish error: {:?}", e);
        };
    }

    Ok(())
}
