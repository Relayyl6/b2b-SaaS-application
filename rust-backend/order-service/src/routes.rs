use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::redis_pub::RedisPublisher;

use crate::models::{CreateOrderRequest, Order, OrderEvent, OrderStatus, UpdateOrderStatus};

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
            INSERT INTO orders (id, user_id, supplier_id, product_id, items, qty, status, expires_at, order_timestamp, version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 1)
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
            redis_pub.publish_async("order.created", event.clone());

            HttpResponse::Created().json(serde_json::json!({
                "message": "Order successfully created",
                "id": order,
            }))
        }
        Err(err) => {
            eprintln!("Error creating order: {}", err);
            HttpResponse::InternalServerError().json(json!({"error": "Failed to create order"}))
        }
    }
}

#[get("/orders/{id}")]
pub async fn get_order(pool: web::Data<PgPool>, path: web::Path<Uuid>) -> HttpResponse {
    let order_id = path.into_inner();
    let result = sqlx::query_as::<_, Order>(
        r#"
            SELECT *
            FROM orders
            WHERE id = $1
        "#,
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
    let expires_at = req
        .expires_at
        .unwrap_or(Utc::now() + Duration::seconds(2 * 24 * 60 * 60));
    let product_id = req.product_id.unwrap_or(Uuid::new_v4());

    // Update status and return the final updated status
    let result = sqlx::query_as::<_, Order>(
        r#"
            UPDATE orders
            SET
                status = COALESCE($1, status),
                order_timestamp = COALESCE($2, order_timestamp),
                expires_at = COALESCE($3, expires_at),
                updated_at = NOW(),
                version = version + 1
            WHERE id = $4 AND product_id = $5 AND user_id = $6
            AND ($7 IS NULL OR version = $7)
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
    .bind(product_id)
    .bind(user_id)
    .bind(req.expected_version)
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
                        order_timestamp: Some(order.order_timestamp),
                    };
                    redis_pub.publish_async("order.cancelled", cancel_event.clone());
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

                OrderStatus::Shipped => {
                    let shipped_event = OrderEvent {
                        event_type: "order.shipped".to_string(),
                        product_id: order.product_id,
                        supplier_id: order.supplier_id,
                        order_id: Some(order.id),
                        quantity: order.qty,
                        user_id: Some(order.user_id),
                        timestamp: order.order_timestamp,
                        ..Default::default()
                    };
                    redis_pub.publish_async("order.shipped", shipped_event);
                    println!("Order {} shipped", order.id);
                }

                OrderStatus::Refunded => {
                    let refunded_event = OrderEvent {
                        event_type: "order.refunded".to_string(),
                        product_id: order.product_id,
                        supplier_id: order.supplier_id,
                        order_id: Some(order.id),
                        quantity: order.qty,
                        user_id: Some(order.user_id),
                        timestamp: order.order_timestamp,
                        ..Default::default()
                    };
                    redis_pub.publish_async("order.refunded", refunded_event);
                    println!("Order {} refunded", order.id);
                }

                OrderStatus::Processing => {
                    println!("Order {} is processing", order.id);
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
        }
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().json(serde_json::json!({"error": "Order not found"}));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": format!("DB Error: {}", e)}));
        }
    }
}

// implement route to delete an order, not: only updating orders, cancelling it and deleting it is allowed.
// TODO: add a order_timestamp to the delete route, after a certain amount o time, orders still pending wil be automatically deleted

#[delete("/orders/{id}/{user_id}")]
pub async fn delete_order(
    _redis_pub: web::Data<RedisPublisher>,
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, Uuid)>,
) -> HttpResponse {
    let (order_id, user_id) = path.into_inner();
    let result = sqlx::query("DELETE FROM orders WHERE id = $1 AND user_id = $2")
        .bind(order_id)
        .bind(user_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(row) if row.rows_affected() > 0 => {
            // redis_pub.publish("order.deleted", &event).await.unwrap();
            HttpResponse::Ok().body("Order deleted successfully")
        }
        Ok(_) => HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            eprintln!("DB error deleting order: {:?}", e);
            HttpResponse::InternalServerError().body("DB error")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_create_and_update_order_optimistic_concurrency(pool: PgPool) {
        let redis_pub = web::Data::new(RedisPublisher::new_noop());
        let pool_data = web::Data::new(pool);
        
        let mut app = test::init_service(
            App::new()
                .app_data(pool_data.clone())
                .app_data(redis_pub.clone())
                .service(create_order)
                .service(update_status)
        ).await;

        let user_id = Uuid::new_v4();
        let supplier_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        // 1. Create order
        let create_req = CreateOrderRequest {
            user_id,
            supplier_id,
            product_id,
            qty: 5,
            status: Some(OrderStatus::Pending),
            items: json!([{"name": "test item"}]),
        };

        let req = test::TestRequest::post()
            .uri("/orders")
            .set_json(&create_req)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        let order_id = Uuid::parse_str(body["id"]["id"].as_str().unwrap()).unwrap();
        let initial_version = body["id"]["version"].as_i64().unwrap() as i32;
        assert_eq!(initial_version, 1);

        // 2. Update order (matching version) - State transition Pending -> Confirmed
        let update_req_success = UpdateOrderStatus {
            id: order_id,
            product_id: Some(product_id),
            user_id: Some(user_id),
            new_status: Some(OrderStatus::Confirmed),
            expires_at: None,
            order_timestamp: None,
            expected_version: Some(initial_version),
        };
        
        let req2 = test::TestRequest::put()
            .uri(&format!("/orders/{}/status", order_id))
            .set_json(&update_req_success)
            .to_request();
        let resp2 = test::call_service(&mut app, req2).await;
        assert_eq!(resp2.status(), actix_web::http::StatusCode::OK);
        
        let body2: serde_json::Value = test::read_body_json(resp2).await;
        let new_version = body2["status"]["version"].as_i64().unwrap() as i32;
        assert_eq!(new_version, 2);
        
        // 3. Update order (mismatched version - Optimistic Concurrency Failure)
        let update_req_fail = UpdateOrderStatus {
            id: order_id,
            product_id: Some(product_id),
            user_id: Some(user_id),
            new_status: Some(OrderStatus::Shipped),
            expires_at: None,
            order_timestamp: None,
            expected_version: Some(1), // old version!
        };
        
        let req3 = test::TestRequest::put()
            .uri(&format!("/orders/{}/status", order_id))
            .set_json(&update_req_fail)
            .to_request();
        let resp3 = test::call_service(&mut app, req3).await;
        assert_eq!(resp3.status(), actix_web::http::StatusCode::NOT_FOUND); // Not found because of WHERE version = $7 mismatch
    }

    #[sqlx::test]
    async fn test_invalid_state_transition(pool: PgPool) {
        let redis_pub = web::Data::new(RedisPublisher::new_noop());
        let pool_data = web::Data::new(pool);
        
        let mut app = test::init_service(
            App::new()
                .app_data(pool_data.clone())
                .app_data(redis_pub.clone())
                .service(create_order)
                .service(update_status)
        ).await;

        let user_id = Uuid::new_v4();
        let supplier_id = Uuid::new_v4();
        let product_id = Uuid::new_v4();

        // 1. Create order (Pending)
        let create_req = CreateOrderRequest {
            user_id,
            supplier_id,
            product_id,
            qty: 1,
            status: Some(OrderStatus::Pending),
            items: json!([]),
        };

        let req = test::TestRequest::post()
            .uri("/orders")
            .set_json(&create_req)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let order_id = Uuid::parse_str(body["id"]["id"].as_str().unwrap()).unwrap();

        // 2. Try invalid transition (Pending -> Delivered)
        let invalid_update = UpdateOrderStatus {
            id: order_id,
            product_id: Some(product_id),
            user_id: Some(user_id),
            new_status: Some(OrderStatus::Delivered),
            expires_at: None,
            order_timestamp: None,
            expected_version: None, // Ignore version for this test
        };
        
        let req2 = test::TestRequest::put()
            .uri(&format!("/orders/{}/status", order_id))
            .set_json(&invalid_update)
            .to_request();
        let resp2 = test::call_service(&mut app, req2).await;
        
        // Should return 404 because the state transition condition in the WHERE clause fails
        assert_eq!(resp2.status(), actix_web::http::StatusCode::NOT_FOUND); 
    }
}


