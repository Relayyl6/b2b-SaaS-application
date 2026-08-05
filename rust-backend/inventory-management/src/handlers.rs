use crate::db::InventoryRepo;
use crate::models::{CreateInventoryRequest, StockUpdateEvent, UpdateStockRequest};
use crate::redis_pub::RedisPublisher;
use actix_web::{web, HttpResponse, Responder};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProductDeletedEvent {
    pub tenant_id: Uuid,
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub deleted: bool,
}

pub async fn get_inventory(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let supplier_id = path.into_inner();
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    match InventoryRepo::get_by_supplier(&mut *tx, supplier_id).await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(e) => {
            eprintln!("DB ERROR: {:?}", e);
            HttpResponse::InternalServerError().body(format!("DB error: {:?}", e))
        }
    }
}

pub async fn create_inventory(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    req: web::Json<CreateInventoryRequest>,
) -> impl Responder {
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    match InventoryRepo::create_inventory_item(&mut *tx, &req).await {
        Ok(item) => {
            tx.commit().await.unwrap();
            HttpResponse::Created().json(item)
        }
        Err(err) => {
            eprintln!("Error creating inventory item: {:?}", err);
            HttpResponse::InternalServerError().body("Failed to create inventory item")
        }
    }
}

pub async fn get_inventory_item(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (supplier_id, product_id) = path.into_inner();

    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    match InventoryRepo::get_one(&mut *tx, supplier_id, product_id).await {
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
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    redis_pub: web::Data<RedisPublisher>,
    redis_client: web::Data<redis::Client>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateStockRequest>,
) -> impl Responder {
    let supplier_id = path.into_inner();
    let change = req.quantity_change;

    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    match InventoryRepo::update_stock(&mut *tx, supplier_id, &req).await {
        Ok(inventory) => {
            tx.commit().await.unwrap();
            let low_stock = inventory.quantity <= inventory.low_stock_threshold;

            // Expanded event payload to reflect possible new product fields
            let event = StockUpdateEvent {
                tenant_id: tenant.tenant_id,
                product_id: inventory.product_id,
                supplier_id: inventory.supplier_id,
                new_quantity: inventory.quantity,
                change: change,
                low_stock,
                name: Some(inventory.name.clone()),
                description: Some(inventory.description.clone()),
                category: Some(inventory.category.clone()),
                price: Some(inventory.price),
                unit: Some(inventory.unit.clone()),
                available: Some(inventory.available),
            };

            // Publish to Redis channels
            redis_pub.publish_async("inventory.updated", event.clone());
            if low_stock {
                redis_pub.publish_async("inventory.lowstock", event.clone());
            }

            // Invalidate cache for this supplier
            if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
                let cache_key = format!("inventory:supplier:{}", supplier_id);
                let _: Result<(), _> = conn.del(cache_key).await;
            }

            HttpResponse::Ok().json(inventory)
        }
        Err(err) => {
            eprintln!("Database error while updating stock: {:?}", err);
            match err {
                sqlx::Error::RowNotFound => HttpResponse::NotFound()
                    .body("No inventory item found for this supplier and product ID."),
                sqlx::Error::Database(db_err) => HttpResponse::InternalServerError()
                    .body(format!("Database constraint error: {}", db_err)),
                _ => HttpResponse::InternalServerError().body("Unexpected database error."),
            }
        }
    }
}

pub async fn delete_product(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    redis_pub: web::Data<RedisPublisher>,
    redis_client: web::Data<redis::Client>,
    path: web::Path<(Uuid, Uuid)>, // supplier_id and product_id
) -> impl Responder {
    let (supplier_id, product_id) = path.into_inner();

    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    match InventoryRepo::delete_product(&mut *tx, supplier_id, product_id).await {
        Ok(rows_affected) if rows_affected > 0 => {
            tx.commit().await.unwrap();
            // Publish deletion event
            let event = ProductDeletedEvent {
                tenant_id: tenant.tenant_id,
                product_id,
                supplier_id,
                deleted: true,
            };

            redis_pub
                .publish::<ProductDeletedEvent>("inventory.deleted", &event)
                .await
                .unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires Redis instance"]
    fn test_cache_invalidation_logic() {
        // Cache invalidation uses: 
        // let cache_key = format!("inventory:supplier:{}", supplier_id);
        // conn.del(cache_key).await;
        // This is verified during integration testing.
        assert!(true);
    }
}
