use chrono::Utc;
use sqlx::PgPool;
use tokio::time::{interval, Duration};
use serde_json::json;

pub async fn start_reservation_expiration_worker(pool: PgPool, redis_pub: RedisPublisher) {
    tokio::spawn(async move {
        let mut interval_timer = interval(Duration::from_secs(30));

        loop {
            interval_timer.tick().await;

            match clean_expired_reservations(&pool, &redis_pub).await {
                Ok(_) => println!("Expired reservation cleanup complete"),
                Err(e) => eprintln!("Failed to clean expired reservations: {:?}", e),
            }
        }
    });
}

async fn clean_expired_reservations(
    pool: &PgPool,
    redis_pub: &RedisPublisher
) -> Result<(), sqlx::Error> {
    let expired = sqlx::query!(
        r#"
            SELECT reservation_id, product_id, order_id, qty, user_id
            FROM reservations
            WHERE expires_at < NOW() AND released = false
            RETURNING *
        "#
    )
    .fetch_all(pool)
    .await?;

    for res in expired {
        // Release stock
        sqlx::query!(
            r#"
                UPDATE inventory
                SET reserved = reserved - $1
                WHERE product_id = $2
            "#,
        )
        .bind(res.qty)
        .bind(res.product_id)
        .execute(pool)
        .await?;

        // Mark reservation expired
        sqlx::query!(
            r#"
                UPDATE reservations
                SET released = true
                WHERE reservation_id = $1
            "#, // released = true basically means status = "expired"
        )
        .bind(res.reservation_id)
        .execute(pool)
        .await?;

        // Publish event to order service
        let event = json!({
            "event_type": "inventory.reservation_expired",
            "id": res.order_id,
            "user_id": res.user_id,
            "product_id": res.product_id,
            "new_status": "failed".to_string(), // failed aka expired
            "timestamp": Utc::now().timestamp_millis(),
        });

        redis_pub.publish("inventory.reservation_expired", &event).await?;
    }

    Ok(())
}
