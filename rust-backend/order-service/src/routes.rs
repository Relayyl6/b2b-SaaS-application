use actix_web::{get, post, put, delete, web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::json;
use chrono::{Duration, Utc};

use crate::redis_pub::RedisPublisher;

use crate::models::{CreateOrderRequest, OrderStatus, OrderEvent, UpdateOrderStatus, Order};

#[post("/orders")]
pub async fn create_order(
    pool: web::Data<PgPool>,
    redis_pub: web::Data<RedisPublisher>,
    req: web::Json<CreateOrderRequest>,
) -> HttpResponse {
    let order_id = Uuid::new_v4();
    let status = req.status.clone().unwrap_or(OrderStatus::Pending);
    let order_timestamp = Utc::now();

    // adjust timing, configurable to add flexibility for when the customer is able to pay
    let expires_at = Utc::now() + Duration::seconds(2 * 24 * 60 * 60);

    let result = sqlx::query_as::<_, Order>(
        r#"
            INSERT INTO orders (id, user_id, supplier_id, product_id, items, qty, status, expires_at, order_timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#
    )
    .bind(&order_id)
    .bind(req.user_id)
    .bind(req.supplier_id)
    .bind(req.product_id)
    .bind(&req.items)
    .bind(req.qty)
    .bind(status)
    .bind(expires_at)
    .bind(order_timestamp)
    .fetch_one(pool.get_ref())
    .await;

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

                // Order-related fields
                order_id: Some(order.id),
                quantity: order.qty,
                reservation_id: None,
                user_id: Some(order.user_id),
                expires_at: order.expires_at,

                // Add order_timestamp for event ordering
                timestamp: order.order_timestamp,

                ..Default::default()
            };


            if let Err(e) = redis_pub.publish("order.created", &event).await {
                eprintln!("Redis publish error (order.created): {:?}", e);
            }

            HttpResponse::Created().json(serde_json::json!({
                "message": "Order successfully created",
                "id": order,
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
    path: web::Path<Uuid>
) -> HttpResponse {
    let order_id = path.into_inner();
    let result = sqlx::query_as::<_, Order>(
        r#"
            SELECT *
            FROM orders
            WHERE id = $1
        "#
    )
    .bind(order_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(order) => HttpResponse::Ok().json(order),
        Err(_) => HttpResponse::NotFound().json(json!({"error": "Order not found"})),
    }
}

#[put("/orders/{id}/status")]
pub async fn update_status(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    redis_pub: web::Data<RedisPublisher>,
    req: web::Json<UpdateOrderStatus>,
) -> HttpResponse {
    let order_id = path.into_inner();
    let new_status = req.new_status.clone().unwrap_or(OrderStatus::Pending);
    let user_id = req.user_id;
    let order_timestamp = req.order_timestamp.unwrap_or(Utc::now());
    let expires_at = req.expires_at.unwrap_or(
        Utc::now() + Duration::seconds(2 * 24 * 60 * 60)
    );
    let product_id = req.product_id.unwrap_or(Uuid::new_v4());

    // 1️⃣ Update status and return the final updated status
    let result = sqlx::query_as::<_, Order>(
        r#"
            UPDATE orders
            SET
                status = COALESCE($1, status),
                order_timestamp = COALESCE($2, order_timestamp),
                expires_at = COALESCE($3, expires_at),
                updated_at = NOW()
            WHERE id = $4 AND product_id = $5 AND user_id = $6
            RETURNING *
        "#
    )
    .bind(new_status)
    .bind(order_timestamp)
    .bind(expires_at)
    .bind(order_id)
    .bind(product_id)
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(order) => {
            match order.status {
                OrderStatus::Failed => {
                    // Trigger notification service (email/SMS)
                    // Optionally notify logistics (shipment preparation)
                    // Notify payments to refund payment if already made
                    // this should be a redis listener for when the order.failed is gotten from inventory management, to update the order's status
                    // publish event to logistics service (inventory management already does that when inventory.reserved, i.e.the product has been confirmed that the order wasnt already existing or the order hasnt expired)
                    // TODO: add a listener for when inventory mangement sends ordr.failed when the product has ben expired or rejected
                    println!("Order {} failed", order.id);
                }

                OrderStatus::Confirmed => {
                    // Trigger notification service (email/SMS)
                    // Optionally notify logistics (shipment preparation)
                    // Log audit entry
                    // this should be a redis listener for when the order.confirmed is gotten from inventory management, to update the order's status
                    // publish event to logistics service (inventory management already does that when inventory.reserved, i.e.the product has been confirmed that the order wasnt already existing or the order hasnt expired)
                    // TODO: add a listener for when inventory mangement sends ordr.confirmed when the roduct has ben reserved
                    println!("Order {} confirmed", order.id);
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
                        order_id: Some(order.id),
                        quantity: order.qty,
                        reservation_id: None,
                        expires_at: order.expires_at,
                        user_id: Some(order.user_id),

                        // Add order_timestamp for event ordering
                        timestamp: order.order_timestamp,
                    };


                    if let Err(e) = redis_pub.publish("order.cancelled", &cancel_event).await {
                        eprintln!("Redis publish error (order.cancelled): {:?}", e);
                    }
                    println!("Order {} cancelled", order.id);
                }

                OrderStatus::Delivered => {
                    // mark delivery order_timestamp, request review, receive event from logistics service
                    // instead of, delete order from order table after a order_timestamp
                    // No.
                    // Do soft delete: ALTER TABLE orders ADD COLUMN deleted_at TIMESTAMPTZ NULL;
                    // Then mark delivered orders as: UPDATE orders SET deleted_at = now() WHERE id = ?

                    println!("Order {} delivered", order.id);
                }

                OrderStatus::Pending => {
                    // maybe nothing
                    // Add expires_at to orders
                    // Run a cron job or background task to auto-expire pending orders
                    // Emit order.expired
                    // Let Inventory handle release
                    // I realise now, if stuff is edited from a pending (not yet confirmed or failed order, then it should publish an event. Inventory checks if its existing already)

                    println!("Order {} set to Pending", order.id);
                }

                _ => {
                    // fallback for new statuses
                        println!("Order {} updated to {:?}", order.id, order.status);
                }
            }

            // Response
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Order status updated",
                "status": order
            }))
        },
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().json(
                serde_json::json!({"error": "Order not found"})
            );
        },
        Err(e) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("DB Error: {}", e)})
            );
        }
    }
}

// implement route to delete an order, not: only updating orders, cancelling it and deleting it is allowed.
// TODO: add a order_timestamp to the delete route, after a certain amount o time, orders still pending wil be automatically deleted

#[delete("/orders/{id}/{user_id}")]
pub async fn delete_order(
    redis_pub: web::Data<RedisPublisher>,
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, Uuid)>
) -> HttpResponse {
    let (order_id, user_id) = path.into_inner();
    let result = sqlx::query!(
        r#"
            DELETE FROM orders WHERE id = $1 AND user_id = $2
        "#,
        order_id,
        user_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(row) if row.rows_affected() > 0 => {
            // redis_pub.publish("order.deleted", &event).await.unwrap();
            HttpResponse::Ok().body("Order deleted successfully")
        },
        Ok(_) => HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            eprintln!("DB error deleting order: {:?}", e);
            HttpResponse::InternalServerError().body("DB error")
        }
    }
}