use actix_web::{get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::json;

use crate::models::{CreateOrderRequest, Order, OrderStatus, OrderEvent};

#[post("/orders")]
pub async fn create_order(
    pool: web::Data<PgPool>,
    req: web::Json<CreateOrderRequest>,
) -> impl Responder {
    let order_id = Uuid::new_v4();
    let status = req.status.clone().unwrap_or(OrderStatus::Pending);
    let qty = req.qty.unwrap_or(0);
    let timestamp = Utc::now().timestamp_millis();
    expires_ttl_secs = 2 * 24 * 60 * 60; // adjust timing, configurable to add flexibility for when the customer is able to pay
    let expires_at = Utc::now() + Duration::seconds(expires_ttl_secs);

    let result = sqlx::query!(
        Order,
        r#"
            INSERT INTO orders (id, user_id, supplier_id, product_id, items, qty, status, expires_at, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#
    )
    .bind(&order_id)
    .bind(req.user_id)
    .bind(req.supplier_id)
    .bind(req.product_id)
    .bind(&req.items)
    .bind(qty)
    .bind(status)
    .bind(expires_at)
    .bind(timestamp)
    .fetch_one(pool.get_ref())
    .await?;

    match result {
        Ok(order) => {
            let event = OrderEvent {
                event_type: "order.created".to_string(),
                product_id: order.product_id,
                supplier_id: order.supplier_id,

                // Product-related fields (None since this event is order-based, their implementation is in product catalog)
                // name: None,
                // description: None,
                // price: None,
                // category: None,
                // low_stock_threshold: None,
                // unit: None,
                // quantity_change: None,
                // available: None,
                ..Default::default(),

                // Order-related fields
                order_id: Some(order.order_id),
                quantity: Some(order.qty),
                reservation_id: None,

                user_id: order.user_id,

                expires_at: order.expires_at,

                // Add timestamp for event ordering
                timestamp: Some(Utc::now().timestamp_millis()),
            };


            if let Err(e) = redis_pub.publish("order.created", &event).await {
                eprintln!("Redis publish error (order.created): {:?}", e);
            }

            HttpResponse::Created().json(serde_json::json!({
                "message": "Order successfully created",
                "id": order_id,
            }))
        },
        Err(err) => {
            eprintln!("Error creating order: {}", err);
            HttpResponse::InternalServerError().json(json!({"error": "Failed to create order"}))
        }
    }
}

#[get("/orders/{id}")]
pub async fn get_order(
    pool: web::Data<PgPool>,
    order_id: web::Path<Uuid>
) -> impl Responder {
    let result = sqlx::query_as!(
        Order,
        r#"
            SELECT *
            FROM orders
            WHERE id = $1
            RETURNING *
        "#
    )
    .bind(order_id)
    .fetch_one(pool.get_ref())
    .await?;

    match result {
        Ok(order) => HttpResponse::Ok().json(order),
        Err(_) => HttpResponse::NotFound().json(json!({"error": "Order not found"})),
    }
}

#[put("/orders/{id}/status")]
pub async fn update_status(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateOrderStatus>,
) -> impl Responder {
    let order_id = path.into_inner();
    let new_status = req.new_status.unwrap_or(OrderStatus::Pending);
    let timestamp = req.timestamp.unwrap_or(Utc::now().timestamp_millis(););

    // 1️⃣ Update status and return the final updated status
    let result = sqlx::query!(
        Order,
        r#"
            UPDATE orders
            SET status = COALESCE($1, status)
            SET items = COALESCE($2, items)
            SET timestamp = COALESCE($3, timestamp)
            SET expires_at = COALESCE($4, expires_at)
            WHERE id = $5
            RETURNING *
        "#
    )
    .bind(new_status)
    .bind(&req.items.unwrap_or("No item name specified".to_string()))
    .bind(timestamp)
    .bind(expires_at)
    .bind(order_id)
    .fetch_one(pool.get_ref())
    .await?;

    match result {
        Ok(Some(order)) => {
            match order.status {
                OrderStatus::Failed => {
                    // Trigger notification service (email/SMS)
                    // Optionally notify logistics (shipment preparation)
                    // Notify payments to refund payment if already made
                    // this should be a redis listener for when the order.failed is gotten from inventory management, to update the order's status
                    // publish event to logistics service (inventory management already does that when inventory.reserved, i.e.the product has been confirmed that the order wasnt already existing or the order hasnt expired)
                    // TODO: add a listener for when inventory mangement sends ordr.failed when the product has ben expired or rejected
                    println!("Order {} confirmed", order.order_id);
                }

                OrderStatus::Confirmed => {
                    // Trigger notification service (email/SMS)
                    // Optionally notify logistics (shipment preparation)
                    // Log audit entry
                    // this should be a redis listener for when the order.confirmed is gotten from inventory management, to update the order's status
                    // publish event to logistics service (inventory management already does that when inventory.reserved, i.e.the product has been confirmed that the order wasnt already existing or the order hasnt expired)
                    // TODO: add a listener for when inventory mangement sends ordr.confirmed when the roduct has ben reserved
                    println!("Order {} confirmed", order.order_id);
                }

                OrderStatus::Cancelled => {
                    // release inventory, refund payment, publish event
                    let cancel_event = OrderEvent {
                        event_type: "order.cancelled".to_string(),
                        product_id: order.product_id,
                        supplier_id: order.supplier_id,

                        // Product-related fields (None since this event is order-based, their implementation is in product catalog)
                        name: None,
                        description: None,
                        price: None,
                        category: None,
                        low_stock_threshold: None,
                        unit: None,
                        quantity_change: None,
                        available: None,

                        // Order-related fields
                        order_id: Some(order.order_id),
                        quantity: Some(order.qty),
                        reservation_id: None,
                        expires_at: None,
                        user_id: Some(order.user_id),

                        // Add timestamp for event ordering
                        timestamp: Some(Utc::now().timestamp_millis()),
                    };


                    if let Err(e) = redis_pub.publish("order.cancelled", &cancel_event).await {
                        eprintln!("Redis publish error (order.cancelled): {:?}", e);
                    }
                    println!("Order {} cancelled", order.order_id);
                }

                OrderStatus::Delivered => {
                    // mark delivery timestamp, request review, receive event from logistics service
                    // instead of, delete order from order table after a timestamp
                    // No.
                    // Do soft delete: ALTER TABLE orders ADD COLUMN deleted_at TIMESTAMPTZ NULL;
                    // Then mark delivered orders as: UPDATE orders SET deleted_at = now() WHERE id = ?

                    println!("Order {} delivered", order.order_id);
                }

                OrderStatus::Pending => {
                    // maybe nothing
                    // Add expires_at to orders
                    // Run a cron job or background task to auto-expire pending orders
                    // Emit order.expired
                    // Let Inventory handle release
                    println!("Order {} set to Pending", order.order_id);
                }

                _ => {
                    // fallback for new statuses
                        println!("Order {} updated to {:?}", order.order_id, order.status);
                    }
                }

                // Response
                HttpResponse::Ok().json(serde_json::json!({
                    "message": "Order status updated",
                    "status": order.status
                }))
            }
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(
                serde_json::json!({"error": "Order not found"})
            );
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("DB Error: {}", e)})
            );
        }
    };



    // 2️⃣ Perform extra steps depending on the new status
    match updated_status {
        
}

// implement route to delete an order, not: only updating orders, cancelling it and deleting it is allowed.
// TODO: add a timestamp to the delete route, after a certain amount o time, orders still pending wil be automatically deleted
