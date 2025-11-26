use sqlx::PgPool;
use uuid::Uuid;
use reqwest::Client;
use std::env;
use serde::{Serialize, Deserialize};
use crate::redis_pub::RedisPublisher;
use tokio;
use chrono::{DateTime, Duration, Utc};
use actix_web::web;
use crate::models::UpdateStockRequest;
use crate::redis_sub::InventoryRepo;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProductEvent {
    pub event_type: String,
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub category: Option<String>,
    pub low_stock_threshold: Option<i32>,
    pub unit: Option<String>,
    pub quantity_change: Option<i32>,
    pub available: Option<bool>,
    // Order-related
    pub order_id: Option<Uuid>,
    pub quantity: Option<i32>,
    pub reservation_id: Option<Uuid>,
    pub order_timestamp: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>
    // pub status: OrderStatus,
}

// #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
// #[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub async fn create_product_from_event(
    _pool: &PgPool,
    event: ProductEvent
) -> Result<(), Box<dyn std::error::Error>> {

    let client = Client::new();
    let service_url = env::var("INVENTORY_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".into());
    let url = format!("{}/inventory", service_url);

    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "product_id": event.product_id,
            "supplier_id": event.supplier_id,
            "quantity": event.quantity.unwrap_or(0),
            "name": event.name.clone().unwrap_or("Unnamed product".to_string()),
            "description": event.description.clone().unwrap_or("No description for this product".to_string()),
            "price": event.price.unwrap_or(0.00),
            "category": event.category.clone().unwrap_or("Unspecified Category".to_string()),
            "low_stock_threshold": event.low_stock_threshold.unwrap_or(5),
            "unit": event.unit.unwrap_or("unit".to_string()),
        }))
        .send()
        .await?;
    
    if resp.status().is_success() {
        println!("✅({}) Created product {:?} via API route", event.event_type, event.name);
    } else {
        eprintln!("❌ Failed to create product: {:?}", resp.text().await?);
    }

    Ok(())
}

pub async fn update_product_from_event(
    _pool: &PgPool,
    event: ProductEvent
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let service_url = env::var("INVENTORY_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".into());
    let url = format!("{}/inventory/{}/update", service_url, event.supplier_id);

    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "product_id": event.product_id,
            "supplier_id": event.supplier_id,
            "name": event.name,
            "description": event.description,
            "price": event.price,
            "category": event.category,
            "unit": event.unit,
            "quantity": event.quantity,
            "low_stock_threshold": event.low_stock_threshold,
            "quantity_change": event.quantity_change,
            "available": event.available,
            // Add more fields if your Inventory Service expects them
        }))
        .send()
        .await?;

    if resp.status().is_success() {
        println!("🔁({}) Updated product {:?} via API route", event.event_type, event.name);
    } else {
        eprintln!("❌ Failed to update product: {:?}", resp.text().await?);
    }

    Ok(())
}

pub async fn delete_product_from_event(
    _pool: &PgPool,
    event: ProductEvent
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let service_url = env::var("INVENTORY_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3002".into());
    let url = format!("{}/inventory/{}/{}", service_url, event.supplier_id, event.product_id);

    let resp = client.delete(&url).send().await?;

    if resp.status().is_success() {
        println!("🗑️({}) Deleted product {} via API route", event.event_type, event.product_id);
    } else {
        eprintln!("❌ Failed to delete product: {:?}", resp.text().await?);
    }

    Ok(())
}

pub async fn reserve_stock_from_order(
    pool: &PgPool,
    redis_pub: web::Data<RedisPublisher>,
    event: ProductEvent
) -> Result<(), Box<dyn std::error::Error>> {

    // check the expiration date of the order
    // Find all expired reservations that have not been released
    let expired_reservations = sqlx::query!(
        r#"
            SELECT reservation_id, order_id, product_id, qty, user_id
            FROM reservations
            WHERE expires_at <= NOW()
            AND released = false
            FOR UPDATE
        "#
    )
    .fetch_all(pool)
    .await?;

    // Process each expired reservation
    for r in expired_reservations {
        // Release inventory
        sqlx::query!(
            r#"
                UPDATE inventory
                SET reserved = reserved - $1
                WHERE product_id = $2
            "#,
            r.qty,
            r.product_id
        )
        .execute(pool)
        .await?;

        // Mark the reservation as released, or call it expired
        sqlx::query!(
            r#"
                UPDATE reservations
                SET released = true
                WHERE reservation_id = $1
            "#,
            r.reservation_id
        )
        .execute(pool)
        .await?;

        // Publish cancellation event
        let cancel_event = ProductEvent {
            event_type: "inventory.expired".into(), // it's meant to be "order.failed", but my order service is listening and i wanted it to hear something different, maybe it'll have other uses subsequently
            product_id: r.product_id,
            order_id: Some(r.order_id),
            quantity: Some(r.qty),
            user_id: Some(r.user_id),
            reservation_id: Some(r.reservation_id),
            order_timestamp: Some(Utc::now()),
            ..Default::default()
        };

        for event in &["inventory.expired"] { // , "order.cancelled"       these two events are analogous
            if let Err(e) = redis_pub.publish(event, &cancel_event).await {
                eprintln!("Redis publish error (expired): {}", e);

                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        }

        println!(
            "Expired reservation {} for order {} was released. its status is 'expired'",
            r.reservation_id, r.order_id
        );
    }

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
        "#
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await {
        tx.commit().await?;
        let success_event = ProductEvent {
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

        for event in &["inventory.reserved"] { // ,  "order.confirmed"
            if let Err(e) = redis_pub.publish(event, &success_event).await {
                eprintln!("Redis publish error (reserved): {}", e);

                 // wait before retrying
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        }

        return Ok(());
    }

    // get quantity as well as reserved, to compare them to see if the requested quantity is less than what is avaialable
    let (qty, reserved) = sqlx::query_as::<_, (i32, i32)>(
        r#"
            SELECT quantity, reserved
            FROM inventory
            WHERE product_id = $1 FOR UPDATE
        "#
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
            event_type: "inventory.rejected".into(),
            product_id: product_id,
            order_id: Some(order_id),
            quantity: Some(qty_requested),
            user_id: Some(user_id),
            order_timestamp: Some(Utc::now()),
            ..Default::default()
        };

        for event in &["inventory.rejected"] { // , "order.failed"
            if let Err(e) = redis_pub.publish(event, &reject_event).await {
                eprintln!("Redis inventory.rejected publish error (insuffieient stock): {}", e);

                 // wait before retrying
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        }

        return Ok(());
    }

    // Reserve stock
    sqlx::query!(
        r#"
            UPDATE inventory
            SET reserved = reserved + $1
            WHERE product_id = $2
        "#,
        qty_requested,
        product_id
    )
    .execute(&mut *tx)
    .await?;

    // insert reservation row (idempotency + expiry)
    let reservation_id = Uuid::new_v4();
    // let user_id = Uuid::new_v4();
    // let expires_at = Utc::now() + Duration::seconds(reservation_ttl_secs);
    sqlx::query!(
        r#"
            INSERT INTO reservations (reservation_id, order_id, product_id, qty, user_id, expires_at, created_at, released)
            VALUES ($1, $2, $3, $4, $5, $6, now(), false)
        "#,
        reservation_id,
        order_id,
        product_id,
        qty_requested,
        user_id,
        expires_at
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Publish success
    let success_event = ProductEvent {
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


    for event in &["inventory.reserved"] { //  , "order.confirmed"
        if let Err(e) = redis_pub.publish(event, &success_event).await {
            eprintln!("Redis publish error (reserved): {}", e);
             // wait before retrying
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }
    }

    println!("Stock Reserved for order {}", order_id);

    Ok(())
}

pub async fn release_stock_from_order(
    pool: &PgPool,
    redis_pub: web::Data<RedisPublisher>,
    event: ProductEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    let product_id = event.product_id;
    let order_id = event.order_id.unwrap_or(Uuid::new_v4());
    let qty = event.quantity.unwrap_or(0);

    let mut tx = pool.begin().await?;

    // Check reservation exists and amount is ok
    let res_row = sqlx::query!(
        r#"
            SELECT reservation_id, qty, released, user_id
            FROM reservations
            WHERE order_id = $1 FOR UPDATE
        "#,
        order_id
    )
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
    let res = sqlx::query!(
        r#"
            UPDATE inventory
            SET reserved = reserved - $1
            WHERE product_id = $2
            AND reserved >= $1
        "#,
        qty,
        product_id
    )
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        tx.rollback().await?;
        return Err("failed to update reserved (insufficient reserved)".into());
    }

    // mark reservation as released
    sqlx::query!(
        r#"
            UPDATE reservations
            SET released = true
            WHERE reservation_id = $1
        "#,
        reservation_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // publish event AFTER commit
    let release_event = ProductEvent {
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

    for event in &["inventory.released"] { // , "order.cancelled"
        if let Err(e) = redis_pub.publish(event, &release_event).await {
            eprintln!("Redis publish error (released): {}", e);
             // wait before retrying
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }
    }

    Ok(())
}

pub async fn finalize_order_after_payment(
    pool: &PgPool,
    redis_pub: web::Data<RedisPublisher>,
    repo: web::Data<InventoryRepo>,
    supplier_id: Uuid,
    event: ProductEvent,
) -> Result<(), Box<dyn std::error::Error>> {

    let order_id = event.order_id.ok_or("missing order_id")?;
    let qty = event.quantity.unwrap_or(0);
    let product_id = event.product_id;

    let mut tx = pool.begin().await?;

    // Fetch reservation
    let res_row = sqlx::query!(
        r#"
        SELECT reservation_id, qty, released, user_id
        FROM reservations
        WHERE order_id = $1
        FOR UPDATE
        "#,
        order_id
    )
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
    sqlx::query!(
        r#"
            UPDATE reservations
                SET released = TRUE
            WHERE reservation_id = $1
        "#,
        reservation_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let update_req = UpdateStockRequest {
        quantity_change: Some(-(qty)), // reduce stock
        ..Default::default()
    };

    repo.update_stock(supplier_id, &update_req).await?;

    let expires_at = Utc::now() + Duration::seconds(2 * 24 * 60 * 60);

    let finalised_event = ProductEvent {
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

    if let Err(e) = redis_pub.publish("inventory.finalized", &finalised_event).await {
        eprintln!("Redis publish error: {}", e);
    }

    Ok(())
}
