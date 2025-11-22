use sqlx::PgPool;
use uuid::Uuid;
use reqwest::Client;
use std::env;
use serde::{Serialize, Deserialize};
use crate::redis_pub::RedisPublisher;
use tokio;

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub timestamp: Option<i64>,
    pub expires_at: Option<i64>,
    pub user_id: Option<Uuid>
    // pub status: OrderStatus,
}

// #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
// #[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub async fn create_product_from_event(_pool: &PgPool, event: ProductEvent) -> Result<(), Box<dyn std::error::Error>> {

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

pub async fn update_product_from_event(_pool: &PgPool, event: ProductEvent) -> Result<(), Box<dyn std::error::Error>> {
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

pub async fn delete_product_from_event(_pool: &PgPool, event: ProductEvent) -> Result<(), Box<dyn std::error::Error>> {
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

        // Mark the reservation as released
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
            event_type: "inventory.expired".into(), // it's meant to be "inventory.expired", but my oder service is listening and i wanted it to hear order.failed 
            product_id: Some(r.product_id),
            order_id: Some(r.order_id),
            quantity: Some(r.qty),
            user_id: Some(r.user_id),
            reservation_id: Some(r.reservation_id),
            timestamp: Some(Utc::now().timestamp_millis()),
            ..Default::default()
        };

        for event in &["inventory.expired", "order.cancelled"] { // these two events are analogous
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

    let product_id = event.product_id.ok_or("Missing product_id")?;
    let order_id = event.order_id.ok_or("Missing order_id")?;
    let qty_requested = event.quantity.ok_or("Missing quantity")?;
    let user_id = event.user_id.ok_or("Missing user_id")?;


    // adjust timing, configurable to add flexibility for when the customer is able to pay
    let expires_at = Utc::now() + Duration::seconds(2 * 24 * 60 * 60);


    // Atomically check & reserve stock
    let mut tx = pool.begin().await?;

    // ensure reservation for this order doesn't already exist (idempotency)
    let existing: Option<(Uuid, i32)> = sqlx::query_as!(
        r#"
            SELECT reservation_id::uuid AS \"reservation_id!: Uuid\", qty 
            FROM reservations
            WHERE order_id = $1
        "#
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some((reservation_id, qty)) = existing {
        tx.commit().await?;
        let success_event = ProductEvent {
            event_type: "inventory.reserved".into(),
            product_id: Some(product_id),
            order_id: Some(order_id),
            quantity: Some(qty),
            user_id: Some(user_id),
            expires_at: Some(expires_at),
            reservation_id: Some(reservation_id),
            timestamp: Some(Utc::now().timestamp_millis()),
            ..Default::default()
        };

        for event in &["inventory.reserved", "order.confirmed"] {
            if let Err(e) = redis_pub.publish(event, &success_event).await {
                eprintln!("Redis publish error (reserved): {}", e);

                 // wait before retrying
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        }

        return Ok(());
    }

    // get quantity as well as reserved if product wasnt already reserved 
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
            product_id: Some(product_id),
            order_id: Some(order_id),
            quantity: Some(qty_requested),
            user_id: Some(user_id),
            timestamp: Some(Utc::now().timestamp_millis())
            ..Default::default()
        };

        for event in &["inventory.rejected", "order.failed"] {
            if let Err(e) = redis_pub.publish(event, &reject_event).await {
                eprintln!("Redis inventory.rejected publish error (reserved): {}", e);

                 // wait before retrying
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        }

        return Ok(());
    }

    // Reserve stock
    sqlx::query(
        r#"
            UPDATE inventory
            SET reserved = reserved + $1
            WHERE product_id = $2
        "#
    )
    .bind(qty_requested)
    .bind(product_id)
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
        "#
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
        event_type: "inventory.reserved".into(),
        product_id: Some(product_id),
        order_id: Some(order_id),
        quantity: Some(qty_requested),
        expires_at: Some(expires_at),
        user_id: Some(user_id),
        reservation_id: Some(reservation_id),
        timestamp: Some(Utc::now().timestamp_millis()),
        ..Default::default()
    };


    for event in &["inventory.reserved", "order.confirmed"] {
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
    event: ProductEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    let product_id = event.product_id.ok_or_else(|| "Missing product_id".into())?;
    let order_id = event.order_id.ok_or_else(|| "Missing order_id".into())?;
    let qty = event.quantity.ok_or_else(|| "Missing quantity".into())?;

    let mut tx = pool.begin().await?;

    // Check reservation exists and amount is ok
    let res_row = sqlx::query!(
        r#"
            SELECT reservation_id, qty, released, user_id
            FROM reservations
            WHERE order_id = $1 FOR UPDATE
        "#
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await?;

    if res_row.is_none() {
        // nothing to release; idempotent success
        tx.rollback().await?;
        return Ok(());
    }

    let reservation_id: Uuid = res_row.unwrap().reservation_id;
    let reserved_qty: i32 = res_row.unwrap().qty;
    let released_flag: bool = res_row.unwrap().released;
    let user_id: Uuid = res_row.unwrap().user_id;

    if released_flag {
        tx.rollback().await?;
        return Ok(()); // already released
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
        "#
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
    sqlx::query!(
        r#"
            UPDATE reservations
            SET released = true
            WHERE reservation_id = $1
        "#
    )
    .bind(reservation_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // publish event AFTER commit
    let release_event = ProductEvent {
        event_type: "inventory.released".into(),
        product_id: Some(product_id),
        order_id: Some(order_id),
        quantity: Some(qty),
        user_id: Some(user_id),
        expires_at: Some(expires_at),
        reservation_id: Some(reservation_id),
        timestamp: Some(Utc::now().timestamp_millis()),
        ..Default::default()
    };

    for event in &["inventory.released", "order.cancelled"] {
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
    event: ProductEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    let product_id = event.product_id.ok_or_else(|| "Missing product_id".into())?;
    let order_id = event.order_id.ok_or_else(|| "Missing order_id".into())?;
    let qty = event.quantity.ok_or_else(|| "Missing quantity".into())?;

    let mut tx = pool.begin().await?;

    // Option A: use reservations table
    let res_row = sqlx::query!(
        r#"
            SELECT reservation_id, qty, released
            FROM reservations
            WHERE order_id = $1 FOR UPDATE
        "#
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await?;

    if res_row.is_none() {
        tx.rollback().await?;
        return Err("No reservation found for order".into());
    }

    let reservation_id: Uuid = res_row.unwrap().reservation_id;
    let reserved_qty: i32 = res_row.unwrap().qty;
    let released_flag: bool = res_row.unwrap().released;

    if released_flag {
        tx.rollback().await?;
        return Err("Reservation already released".into());
    }

    if qty > reserved_qty {
        tx.rollback().await?;
        return Err("Payment quantity exceeds reserved".into());
    }

    // Atomically decrement both reserved and quantity; ensure reserved >= qty
    let res = sqlx::query!(
        r#"
            UPDATE inventory
            SET reserved = reserved - $1, quantity = quantity - $1
            WHERE product_id = $2
            AND reserved >= $1
            AND quantity >= $1
        "#
    )
    .bind(qty)
    .bind(product_id)
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        tx.rollback().await?;
        return Err("failed to finalize: insufficient numbers".into());
    }

    // mark reservation consumed
    sqlx::query!(
        r#"
            UPDATE reservations
            SET released = true
            WHERE reservation_id = $1
        "#
    )
    .bind(reservation_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // publish event
    let finalised_event = ProductEvent {
        event_type: "inventory.finalized".into(),
        product_id: Some(product_id),
        order_id: Some(order_id),
        quantity: Some(qty),
        user_id: Some(user_id),
        expires_at: Some(expires_at),
        reservation_id: Some(reservation_id),
        timestamp: Some(Utc::now().timestamp_millis()),
        ..Default::default()
    };

    for event in &["inventory.finalized", "order.shipped"] {
        if let Err(e) = redis_pub.publish(event, &finalised_event).await {
            eprintln!("Redis publish error (finalized): {}", e);
             // wait before retrying
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }
    }
    
    Ok(())
}
