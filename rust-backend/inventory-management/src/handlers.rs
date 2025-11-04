use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;
use crate::models::{UpdateStockRequest, StockUpdateEvent, CreateInventoryRequest};
use crate::db::InventoryRepo;
use crate::redis_pub::RedisPublisher;
use redis::AsyncCommands;


// #[derive(Deserialize)]
// pub struct UpdateQuantity {
//     product_id: Uuid,
//     quantity_change: i32,
// }

pub async fn get_inventory(
    repo: web::Data<InventoryRepo>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let supplier_id = path.into_inner();
    match repo.get_by_supplier(supplier_id).await {
        Ok(items) => HttpResponse::Ok().json(items),
        // Err(_) => HttpResponse::InternalServerError().body("DB error"),
        Err(e) => {
            eprintln!("DB ERROR: {:?}", e);
            HttpResponse::InternalServerError().body(format!("DB error: {:?}", e))
        }
    }
}

pub async fn create_inventory(
    repo: web::Data<InventoryRepo>,
    req: web::Json<CreateInventoryRequest>,
) -> impl Responder {
    match repo.create_inventory_item(&req).await {
        Ok(item) => HttpResponse::Created().json(item),
        Err(err) => {
            eprintln!("Error creating inventory item: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to create inventory item")
        }
    }
}

pub async fn get_inventory_item(
    repo: web::Data<InventoryRepo>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (supplier_id, product_id) = path.into_inner();

    match repo.get_one(supplier_id, product_id).await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().body("Product not found for this supplier.")
        }
        Err(err) => {
            eprintln!("DB error fetching inventory item: {:?}", err);
            HttpResponse::InternalServerError().body("Database error while fetching item.")
        }
    }
}


pub async fn update_stock(
    repo: web::Data<InventoryRepo>,
    redis_pub: web::Data<RedisPublisher>,
    redis_client: web::Data<redis::Client>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateStockRequest>,
) -> impl Responder {
    let supplier_id = path.into_inner();
    let change = req.quantity_change;

    match repo.update_stock(supplier_id, &req).await {
        Ok(inventory) => {
            let low_stock = inventory.quantity <= inventory.low_stock_threshold;
            let event = StockUpdateEvent {
                product_id: inventory.product_id,
                supplier_id: inventory.supplier_id,
                new_quantity: inventory.quantity,
                change,
                low_stock,
            };

            // Publish to Redis channels
            if let Err(e) = redis_pub.publish(&event, "inventory.updated").await {
                eprintln!("Redis publish error (updated): {}", e);
            }
            if low_stock {
                if let Err(e) = redis_pub.publish(&event, "inventory.lowstock").await {
                    eprintln!("Redis publish error (lowstock): {}", e);
                }
            }

            // Invalidate cache
            if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
                let cache_key = format!("inventory:supplier:{}", supplier_id);
                let _: Result<(), _> = conn.del(cache_key).await;
            }

            HttpResponse::Ok().json(inventory)
        }
        Err(err) => {
            eprintln!("Database error while updating stock: {:?}", err);
            match err {
                sqlx::Error::RowNotFound => {
                    HttpResponse::NotFound().body("No inventory item found for this supplier and product ID.")
                }
                sqlx::Error::Database(db_err) => {
                    HttpResponse::InternalServerError()
                        .body(format!("Database constraint error: {}", db_err))
                }
                _ => HttpResponse::InternalServerError().body("Unexpected database error."),
            }
        }
    }
}

pub async fn delete_product(
    repo: web::Data<InventoryRepo>,
    redis_pub: web::Data<RedisPublisher>,
    redis_client: web::Data<redis::Client>,
    path: web::Path<(Uuid, Uuid)>, // supplier_id and product_id
) -> impl Responder {
    let (supplier_id, product_id) = path.into_inner();

    match repo.delete_product(supplier_id, product_id).await {
        Ok(rows_affected) if rows_affected > 0 => {
            // Publish deletion event
            let event = serde_json::json!({
                "product_id": product_id,
                "supplier_id": supplier_id,
                "deleted": true
            });

            redis_pub.publish(&event, "inventory.deleted").await.unwrap();

            // Invalidate cache
            // let mut conn = redis_client.get_multiplexed_async_connection().await.unwrap();
            // let cache_key = format!("inventory:supplier:{}", supplier_id);
            // let _: () = conn.del(cache_key).await.unwrap();

            // Invalidate cache
            if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
                let cache_key = format!("inventory:supplier:{}", supplier_id);
                let _: Result<(), _> = conn.del(cache_key).await;
            }

            HttpResponse::Ok().body("Product deleted successfully")
        }
        Ok(_) => HttpResponse::NotFound().body("Product not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to delete product"),
    }
}
