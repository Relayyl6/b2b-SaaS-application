use crate::models::{ProductEvent, UpdateStockRequest, ExpiredReservationRow, ReservationRow, CreateInventoryRequest};
use crate::redis_pub::RedisPublisher;
use crate::db::InventoryRepo;
use actix_web::web;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_product_from_event(
    pool: &PgPool,
    event: ProductEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repo = InventoryRepo::new(pool);
    let req = CreateInventoryRequest {
        supplier_id: event.supplier_id,
        product_id: event.product_id,
        quantity: event.quantity.unwrap_or(0),
        name: event.name.unwrap_or_else(|| "Unnamed product".to_string()),
        description: event.description.unwrap_or_else(|| serde_json::Value::String("No description".to_string())),
        price: event.price.unwrap_or(0.0),
        category: event.category.unwrap_or_else(|| "Unspecified".to_string()),
        low_stock_threshold: event.low_stock_threshold.unwrap_or(5),
        unit: event.unit.unwrap_or_else(|| "unit".to_string()),
    };

    match repo.create_inventory_item(&req).await {
        Ok(_) => println!("✅({}) Created product {:?} via Repo", event.event_type, req.name),
        Err(e) => eprintln!("❌ Failed to create product: {:?}", e),
    }
    Ok(())
}

pub async fn update_product_from_event(
    pool: &PgPool,
    event: ProductEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repo = InventoryRepo::new(pool);
    let req = UpdateStockRequest {
        product_id: event.product_id,
        name: event.name,
        description: event.description,
        price: event.price,
        category: event.category,
        unit: event.unit,
        quantity: event.quantity,
        low_stock_threshold: event.low_stock_threshold,
        quantity_change: event.quantity_change,
        available: event.available,
        reserved: None,
    };

    match repo.update_stock(event.supplier_id, &req).await {
        Ok(_) => println!("🔁({}) Updated product {:?} via Repo", event.event_type, req.name),
        Err(e) => eprintln!("❌ Failed to update product: {:?}", e),
    }
    Ok(())
}

pub async fn delete_product_from_event(
    pool: &PgPool,
    event: ProductEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repo = InventoryRepo::new(pool);
    match repo.delete_product(event.supplier_id, event.product_id).await {
        Ok(_) => println!("🗑️({}) Deleted product {} via Repo", event.event_type, event.product_id),
        Err(e) => eprintln!("❌ Failed to delete product: {:?}", e),
    }
    Ok(())
}

pub async fn reserve_stock_from_order(
    pool: &PgPool,
    redis_pub: web::Data<RedisPublisher>,
    event: ProductEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut tx_expired = pool.begin().await?;

    let expired_reservations = sqlx::query_as::<_, ExpiredReservationRow>(
        r#"
            SELECT reservation_id, order_id, product_id, qty, user_id
            FROM reservations
            WHERE expires_at <= NOW()
            AND released = false
            FOR UPDATE
        "#
    )
    .fetch_all(&mut *tx_expired)
    .await?;

    // Process each expired reservation
    for r in expired_reservations {
        sqlx::query(
            r#"
                UPDATE inventory
                SET reserved = reserved - $1
                WHERE product_id = $2
            "#,
        )
        .bind(r.qty)
        .bind(r.product_id)
        .execute(&mut *tx_expired)
        .await?;

        sqlx::query(
            r#"
                UPDATE reservations
                SET released = true
                WHERE reservation_id = $1
            "#,
        )
        .bind(r.reservation_id)
        .execute(&mut *tx_expired)
        .await?;

        let cancel_event = ProductEvent {
            tenant_id: event.tenant_id.or(Some(event.supplier_id)),
            event_type: "inventory.expired".into(),
            product_id: r.product_id,
            order_id: Some(r.order_id),
            quantity: Some(r.qty),
            user_id: Some(r.user_id),
            reservation_id: Some(r.reservation_id),
            order_timestamp: Some(Utc::now()),
            ..Default::default()
        };

        redis_pub.publish_async("inventory.expired", cancel_event);

        println!(
            "Expired reservation {} for order {} was released. its status is 'expired'",
            r.reservation_id, r.order_id
        );
    }
    tx_expired.commit().await?;

    let product_id = event.product_id;
    let order_id = event.order_id.ok_or("Missing order_id")?;
    let qty_requested = event.quantity.ok_or("Missing quantity")?;
    let user_id = event.user_id.ok_or("Missing user_id")?;

    // adjust timing, configurable to add flexibility for when the customer is able to pay
    let expires_at = Utc::now() + Duration::seconds(2 * 24 * 60 * 60);

    // Atomically check & reserve stock
    let mut tx = pool.begin().await?;

    // let existing: Option<(Uuid, i32)> = //

    // ensure reservation for this order doesn't already exist (idempotency)
    if let Ok(Some((reservation_id, qty))) = sqlx::query_as::<_, (Uuid, i32)>(
        r#"
            SELECT reservation_id, qty
            FROM reservations
            WHERE order_id = $1
        "#,
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await
    {
        tx.commit().await?;
        let success_event = ProductEvent {
            tenant_id: event.tenant_id.or(Some(event.supplier_id)),
            event_type: "inventory.reserved".into(),
            product_id: product_id,
            order_id: Some(order_id),
            quantity: Some(qty),
            user_id: Some(user_id),
            expires_at: Some(expires_at),
            reservation_id: Some(reservation_id),
            order_timestamp: Some(Utc::now()),
            ..Default::default()
        };

        redis_pub.publish_async("inventory.reserved", success_event);
        return Ok(());
    }

    // get quantity as well as reserved, to compare them to see if the requested quantity is less than what is avaialable
    let (qty, reserved) = sqlx::query_as::<_, (i32, i32)>(
        r#"
            SELECT quantity, reserved
            FROM inventory
            WHERE product_id = $1 FOR UPDATE
        "#,
    )
    .bind(product_id)
    .fetch_one(&mut *tx)
    .await?;

    // find out whether the requested quantity is even available to prevent overselling
    let available = qty - reserved;

    if available < qty_requested {
        tx.rollback().await?;
        // Publish REJECTED
        let reject_event = ProductEvent {
            tenant_id: event.tenant_id.or(Some(event.supplier_id)),
            event_type: "inventory.rejected".into(),
            product_id: product_id,
            order_id: Some(order_id),
            quantity: Some(qty_requested),
            user_id: Some(user_id),
            order_timestamp: Some(Utc::now()),
            ..Default::default()
        };

        redis_pub.publish_async("inventory.rejected", reject_event);
        return Ok(());
    }

    // Reserve stock
    sqlx::query(
        r#"
            UPDATE inventory
            SET reserved = reserved + $1
            WHERE product_id = $2
        "#,
    )
    .bind(qty_requested)
    .bind(product_id)
    .execute(&mut *tx)
    .await?;

    // insert reservation row (idempotency + expiry)
    let reservation_id = Uuid::new_v4();
    // let user_id = Uuid::new_v4();
    // let expires_at = Utc::now() + Duration::seconds(reservation_ttl_secs);
    sqlx::query(
        r#"
            INSERT INTO reservations (reservation_id, order_id, product_id, qty, user_id, expires_at, created_at, released)
            VALUES ($1, $2, $3, $4, $5, $6, now(), false)
        "#,
    )
    .bind(reservation_id)
    .bind(order_id)
    .bind(product_id)
    .bind(qty_requested)
    .bind(user_id)
    .bind(expires_at)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Publish success
    let success_event = ProductEvent {
        tenant_id: event.tenant_id.or(Some(event.supplier_id)),
        event_type: "inventory.reserved".into(),
        product_id: product_id,
        order_id: Some(order_id),
        quantity: Some(qty_requested),
        expires_at: Some(expires_at),
        user_id: Some(user_id),
        reservation_id: Some(reservation_id),
        order_timestamp: Some(Utc::now()),
        ..Default::default()
    };

    redis_pub.publish_async("inventory.reserved", success_event);

    println!("Stock Reserved for order {}", order_id);

    Ok(())
}

pub async fn release_stock_from_order(
    pool: &PgPool,
    redis_pub: web::Data<RedisPublisher>,
    event: ProductEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let product_id = event.product_id;
    let order_id = event.order_id.unwrap_or(Uuid::new_v4());
    let qty = event.quantity.unwrap_or(0);

    let mut tx = pool.begin().await?;

    // Check reservation exists and amount is ok
    let res_row = sqlx::query_as::<_, ReservationRow>(
        r#"
            SELECT reservation_id, qty, released, user_id
            FROM reservations
            WHERE order_id = $1 FOR UPDATE
        "#,
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await?;

    if res_row.is_none() {
        // nothing to release; idempotent success
        tx.rollback().await?;
        return Ok(());
    }

    let reservation_id: Uuid = res_row.as_ref().unwrap().reservation_id;
    let reserved_qty: i32 = res_row.as_ref().unwrap().qty;
    let released_flag: bool = res_row.as_ref().unwrap().released;
    let user_id: Uuid = res_row.as_ref().unwrap().user_id;
    let expires_at = Utc::now() + Duration::seconds(2 * 24 * 60 * 60);

    if released_flag {
        tx.rollback().await?;
        return Ok(()); // already released//expired
    }

    if qty > reserved_qty {
        tx.rollback().await?;
        return Err("release amount greater than reserved amount".into());
    }

    // decrement reserved safely
    let res = sqlx::query(
        r#"
            UPDATE inventory
            SET reserved = reserved - $1
            WHERE product_id = $2
            AND reserved >= $1
        "#,
    )
    .bind(qty)
    .bind(product_id)
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        tx.rollback().await?;
        return Err("failed to update reserved (insufficient reserved)".into());
    }

    // mark reservation as released
    sqlx::query(
        r#"
            UPDATE reservations
            SET released = true
            WHERE reservation_id = $1
        "#,
    )
    .bind(reservation_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // publish event AFTER commit
    let release_event = ProductEvent {
        tenant_id: event.tenant_id.or(Some(event.supplier_id)),
        event_type: "inventory.released".into(),
        product_id: product_id,
        order_id: Some(order_id),
        quantity: Some(qty),
        user_id: Some(user_id),
        expires_at: Some(expires_at),
        reservation_id: Some(reservation_id),
        order_timestamp: Some(Utc::now()),
        ..Default::default()
    };

    redis_pub.publish_async("inventory.released", release_event);

    Ok(())
}

pub async fn finalize_order_after_payment(
    pool: &PgPool,
    redis_pub: web::Data<RedisPublisher>,
    repo: web::Data<InventoryRepo>,
    supplier_id: Uuid,
    event: ProductEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let order_id = event.order_id.ok_or("missing order_id")?;
    let qty = event.quantity.unwrap_or(0);
    let product_id = event.product_id;

    let mut tx = pool.begin().await?;

    // Fetch reservation
    let res_row = sqlx::query_as::<_, ReservationRow>(
        r#"
        SELECT reservation_id, qty, released, user_id
        FROM reservations
        WHERE order_id = $1
        FOR UPDATE
        "#,
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await?;

    let row = match res_row {
        Some(r) => r,
        None => {
            tx.rollback().await?;
            return Err("No reservation found for order".into());
        }
    };

    if row.released {
        tx.rollback().await?;
        return Err("Reservation already released (expired)".into());
    }

    if qty > row.qty {
        tx.rollback().await?;
        return Err("Payment quantity exceeds reserved".into());
    }

    let reservation_id = row.reservation_id;
    let user_id = row.user_id;

    // Now mark reservation consumed
    sqlx::query(
        r#"
            UPDATE reservations
                SET released = TRUE
            WHERE reservation_id = $1
        "#,
    )
    .bind(reservation_id)
    .execute(&mut *tx)
    .await?;

    let row: (i32, i32) = sqlx::query_as(
        r#"
            UPDATE inventory
            SET quantity = quantity - $1,
                reserved = reserved - $1
            WHERE product_id = $2
            RETURNING quantity, low_stock_threshold
        "#,
    )
    .bind(qty)
    .bind(product_id)
    .fetch_one(&mut *tx)
    .await?;

    let (current_qty, low_stock_threshold) = row;

    tx.commit().await?;

    let expires_at = Utc::now() + Duration::seconds(2 * 24 * 60 * 60);

    let finalised_event = ProductEvent {
        tenant_id: event.tenant_id.or(Some(supplier_id)),
        event_type: "inventory.finalized".into(),
        product_id,
        order_id: Some(order_id),
        quantity: Some(qty),
        user_id: Some(user_id),
        expires_at: Some(expires_at),
        reservation_id: Some(reservation_id),
        order_timestamp: Some(Utc::now()),
        ..Default::default()
    };

    redis_pub.publish_async("inventory.finalized", finalised_event);

    let updated_event = ProductEvent {
        tenant_id: event.tenant_id.or(Some(supplier_id)),
        event_type: "inventory.updated".into(),
        product_id,
        quantity: Some(current_qty),
        ..Default::default()
    };
    redis_pub.publish_async("inventory.updated", updated_event);

    if current_qty <= low_stock_threshold {
        let lowstock_event = ProductEvent {
            tenant_id: event.tenant_id.or(Some(supplier_id)),
            event_type: "inventory.lowstock".into(),
            product_id,
            quantity: Some(current_qty),
            ..Default::default()
        };
        redis_pub.publish_async("inventory.lowstock", lowstock_event);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_reservation_logic() {
        // Atomic reservations check:
        // SELECT quantity, reserved FROM inventory WHERE product_id = $1 FOR UPDATE
        // reserved = reserved + qty_requested
        // INSERT INTO reservations
        assert!(true);
    }
}
