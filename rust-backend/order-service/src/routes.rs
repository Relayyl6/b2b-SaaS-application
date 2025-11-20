use actix_web::{get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::json;

use crate::models::{CreateOrderRequest, Order, OrderStatus};

#[post("/orders")]
pub async fn create_order(
    pool: web::Data<PgPool>,
    req: web::Json<CreateOrderRequest>,
) -> impl Responder {
    let order_id = Uuid::new_v4();
    let status = req.status.clone().unwrap_or(OrderStatus::Pending);
    let qty = req.qty.unwrap_or(0);

    let result = sqlx::query!(
        r#"
            INSERT INTO orders (id, user_id, supplier_id, product_id, items, qty, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#
    )
    .bind(&order_id)
    .bind(req.user_id)
    .bind(req.supplier_id)
    .bind(req.product_id)
    .bind(&req.items)
    .bind(qty)
    .bind(status)
    .fetch_one(pool.get_ref())
    .await?;

    match result {
        Ok(order) => {
            let event = OrderEvent {
                event_type: "order.created".to_string(),
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

                // Add timestamp for event ordering
                timestamp: Some(Utc::now().timestamp_millis()),
            };


            if let Err(e) = redis_pub.publish("order.created", &event).await {
                eprintln!("Redis publish error (order.created): {:?}", e);
            }

            HttpResponse::Created().json(serde_json::json!({
                "message": "Order seccessfully pending"
                "id": order_id
            })),
        }
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
        r#"
            SELECT id, user_id, supplier_id, product_id, items, qty, status
            FROM orders
            WHERE id = $1
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
    let new_status = req.new_status.unwrap_or(OrderStatus::Pending)

    // 1️⃣ Update status and return the final updated status
    let result = sqlx::query!(
        r#"
            UPDATE orders
            SET status = COALESCE($1, status)
            WHERE id = $2
        "#
    )
    .bind(status)
    .bind(order_id)
    .execute(pool.get_ref())
    .await;

    let updated_status = match updated {
        Ok(Some(row)) => row.status,
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
        OrderStatus::Confirmed => {
            // Trigger notification service (email/SMS)
            // Optionally notify logistics (shipment preparation)
            // Log audit entry
            // this should be a redis listener for when the order.confirmed is gotten from inventory management, to update the order's status
            // publish event to logistics service (inventory management already does that when inventory.reserved, i.e.the product has been confirmed that the order wasnt already existing or the order hasnt expired)
            // TODO: add a listener for when inventory mangement sends ordr.confirmed when the roduct has ben reserved
            println!("Order {} confirmed", order_id);
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

                // Add timestamp for event ordering
                timestamp: Some(Utc::now().timestamp_millis()),
            };


            if let Err(e) = redis_pub.publish("order.cancelled", &cancel_event).await {
                eprintln!("Redis publish error (product.created): {:?}", e);
            }
            println!("Order {} cancelled", order_id);
        }

        OrderStatus::Delivered => {
            // mark delivery timestamp, request review, receive event from logistics service
            println!("Order {} delivered", order_id);
        }

        OrderStatus::Pending => {
            // maybe nothing
            println!("Order {} set to Pending", order_id);
        }

        _ => {
            // fallback for new statuses
            println!("Order {} updated to {:?}", order_id, updated_status);
        }
    }

    // Response
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Order status updated",
        "status": updated_status
    }))
}

// implement route to delete an order, not: only updating orders, cancelling it and deleting it is allowed.
// TODO: add a timestamp to the delete route, after a certain amount o time, orders still pending wil be automatically deleted
