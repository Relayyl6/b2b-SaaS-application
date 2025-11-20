use actix_web::{get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::json;

use crate::models::{CreateOrderRequest, Order, UpdateOrderStatus};

#[post("/orders")]
pub async fn create_order(
    pool: web::Data<PgPool>,
    req: web::Json<CreateOrderRequest>,
) -> impl Responder {
    let order_id = Uuid::new_v4();
    let status = req.status.clone().unwrap_or(UpdateOrderStatus::Pending);
    let qty = req.qty.unwrap_or(0);

    let result = sqlx::query!(
        r#"
            INSERT INTO orders (id, restaurant_id, supplier_id, items, qty, status)
            VALUES ($1, $2, $3, $4, $5)
        "#
    )
    .bind(&order_id)
    .bind(req.restaurant_id)
    .bind(req.supplier_id)
    .bind(req.items)
    .bind(qty)
    .bind(status)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Created().json(json!({ "id": order_id })),
        Err(err) => {
            eprintln!("Error creating order: {}", err);
            HttpResponse::InternalServerError().json(json!({"error": "Failed to create order"}))
        }
    }
}

#[get("/orders/{id}")]
pub async fn get_order(pool: web::Data<PgPool>, order_id: web::Path<Uuid>) -> impl Responder {
    let result = sqlx::query_as!(
        Order,
        r#"
            SELECT id, restaurant_id, supplier_id, items, status
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
    order_id: web::Path<Uuid>,
    new_status: web::Json<UpdateOrderStatus>,
) -> impl Responder {
    let result = sqlx::query!(
        r#"
            UPDATE orders
            SET status = $1
            WHERE id = $2
        "#
    )
    .bind(new_status)
    .bind(order_id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(p) => HttpResponse::Ok().json(json!({
            "message": "Order Status updated", 
            "Status": p
        })),
        Err(_) => HttpResponse::InternalServerError().json(json!({"error": "Failed to update"})),
    }
}
