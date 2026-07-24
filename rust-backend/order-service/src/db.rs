use dotenvy::dotenv;
use sqlx::{Pool, Postgres};
use std::env;

pub async fn get_db_pool() -> Pool<Postgres> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres")
}

pub async fn update_order_status_db(
    pool: &sqlx::PgPool,
    order_id: uuid::Uuid,
    new_status: crate::models::OrderStatus,
    expected_version: Option<i32>,
    order_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<crate::models::Order, sqlx::Error> {
    sqlx::query_as::<_, crate::models::Order>(
        r#"
            UPDATE orders
            SET
                status = $1,
                order_timestamp = COALESCE($2, order_timestamp),
                expires_at = COALESCE($3, expires_at),
                updated_at = NOW(),
                version = version + 1
            WHERE id = $4
            AND ($5 IS NULL OR version = $5)
            AND (
                ($1 = 'pending') OR
                ($1 = 'confirmed' AND status = 'pending') OR
                ($1 = 'failed' AND status = 'pending') OR
                ($1 = 'cancelled' AND status != 'cancelled' AND status != 'delivered') OR
                ($1 = 'shipped' AND (status = 'confirmed' OR status = 'processing')) OR
                ($1 = 'delivered' AND status = 'shipped') OR
                ($1 = 'refunded' AND status != 'refunded') OR
                ($1 = 'processing' AND status = 'confirmed')
            )
            RETURNING *
        "#,
    )
    .bind(new_status)
    .bind(order_timestamp)
    .bind(expires_at)
    .bind(order_id)
    .bind(expected_version)
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_update_order_status_db_optimistic_concurrency(pool: PgPool) {
        // Setup initial order
        let order_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let supplier_id = uuid::Uuid::new_v4();
        let product_id = uuid::Uuid::new_v4();
        let items = serde_json::json!([]);

        sqlx::query(
            r#"
            INSERT INTO orders (id, user_id, supplier_id, product_id, items, qty, status, expires_at, order_timestamp, version)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', NOW() + INTERVAL '1 day', NOW(), 1)
            "#
        )
        .bind(&order_id)
        .bind(&user_id)
        .bind(&supplier_id)
        .bind(&product_id)
        .bind(&items)
        .bind(1)
        .execute(&pool)
        .await
        .unwrap();

        // Test valid state transition and valid version (Pending -> Confirmed)
        let order = update_order_status_db(
            &pool, 
            order_id, 
            crate::models::OrderStatus::Confirmed, 
            Some(1), 
            None, 
            None
        ).await.expect("Valid transition should succeed");

        assert_eq!(order.version, 2);
        assert_eq!(order.status, crate::models::OrderStatus::Confirmed);

        // Test invalid version update (version is now 2, we expect 1)
        let err = update_order_status_db(
            &pool, 
            order_id, 
            crate::models::OrderStatus::Shipped, 
            Some(1), // Mismatched version
            None, 
            None
        ).await.expect_err("Should fail due to version mismatch");
        
        assert!(matches!(err, sqlx::Error::RowNotFound));
    }
}



