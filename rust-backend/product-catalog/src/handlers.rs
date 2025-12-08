use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;
use redis::AsyncCommands;
use crate::db::ProductRepo;
use crate::models::{CreateProductRequest, UpdateProductRequest, BulkCreateRequest, ProductEvent};
// use crate::models::Product;
use crate::redis_pub::RedisPublisher;
use sqlx;
use redis;
use serde_json::json;

pub async fn create_product(
    repo: web::Data<ProductRepo>,
    redis_pub: web::Data<RedisPublisher>,
    req: web::Json<CreateProductRequest>,
) -> impl Responder {
    match repo.create_product(&req).await {
        Ok(product) => {
            // publish event
            let event = ProductEvent {
                event_type: "product.created".to_string(),
                product_id: product.product_id,
                supplier_id: product.supplier_id,
                price: Some(product.price),
                category: Some(product.category.clone()),
                name: Some(product.name.clone()),
                description: product.description.clone(),
                quantity: Some(product.quantity),
                low_stock_threshold: None,
                unit: Some(product.unit.clone()),
                quantity_change: None,
                ..Default::default()
            };

            if let Err(e) = redis_pub.publish("product.created", &event).await {
                eprintln!("Redis publish error (product.created): {:?}", e);
            }
            HttpResponse::Created().json(product)
        }
        Err(e) => {
            eprintln!("Create product DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to create product")
        }
    }
}

pub async fn get_products_for_supplier(
    repo: web::Data<ProductRepo>,
    redis_pub: web::Data<RedisPublisher>
    path: web::Path<Uuid>,
) -> impl Responder {
    let supplier_id = path.into_inner();
    match repo.get_by_supplier(supplier_id).await {
        Ok(items) => {
            // publish event
            for item in items {
                println!("{}", item);
                let event = ProductEvent {
                    event_type: "product.viewed".to_string(),
                    product_id: item.product_id,
                    supplier_id: item.supplier_id,
                    price: Some(item.price),
                    category: Some(item.category.clone()),
                    name: Some(item.name.clone()),
                    description: item.description.clone(),
                    quantity: Some(item.quantity),
                    low_stock_threshold: None,
                    unit: Some(item.unit.clone()),
                    quantity_change: None,
                    ..Default::default()
                };

                if let Err(e) = redis_pub.publish("product.viewed", &event).await {
                    eprintln!("Redis publish error (product.viewed): {:?}", e);
                }
            }
            HttpResponse::Ok().json(items)
        }
        Err(e) => {
            eprintln!("Get products DB error: {:?}", e);
            HttpResponse::InternalServerError().body("DB error")
        }
    }
}

pub async fn get_single_product(
    repo: web::Data<ProductRepo>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (supplier_id, product_id) = path.into_inner();
    match repo.get_one(supplier_id, product_id).await {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("DB error")
        }
    }
}

pub async fn update_product(
    repo: web::Data<ProductRepo>,
    redis_pub: web::Data<RedisPublisher>,
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<UpdateProductRequest>,
) -> impl Responder {
    let (supplier_id, product_id) = path.into_inner();

    // If quantity_change is present, adjust quantity field accordingly before update
    let mut update_data = req.into_inner();

    if let Some(_change) = update_data.quantity_change {
        // If quantity_change is set, ignore explicit quantity — it’ll be handled in SQL logic already
        update_data.quantity = None;
    }

    match repo.update_product(supplier_id, product_id, &update_data).await {
        Ok(p) => {
            let event = ProductEvent {
                event_type: "product.updated".to_string(),
                product_id: p.product_id,
                supplier_id: p.supplier_id,
                name: Some(p.name.clone()),
                description: p.description.clone(),
                price: Some(p.price),
                category: Some(p.category.clone()),
                quantity: Some(p.quantity),
                low_stock_threshold: Some(p.low_stock_threshold),
                unit: Some(p.unit.clone()),
                quantity_change: update_data.quantity_change,
                available: Some(p.available),
                ..Default::default()
            };

            if let Err(e) = redis_pub.publish("product.updated", &event).await {
                eprintln!("Redis publish error (product.updated): {:?}", e);
            }

            HttpResponse::Ok().json(p)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            eprintln!("DB error updating product: {:?}", e);
            HttpResponse::InternalServerError().body("DB error")
        }
    }
}


pub async fn delete_product(
    repo: web::Data<ProductRepo>,
    redis_pub: web::Data<RedisPublisher>,
    redis_client: web::Data<redis::Client>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (supplier_id, product_id) = path.into_inner();
    match repo.delete_product(supplier_id, product_id).await {
        Ok(rows) if rows > 0 => {
            let event = json!({
                "event_type": "product.deleted",
                "product_id": product_id,
                "supplier_id": supplier_id,
                "name": "deleted", // either this or scatter the Inventory_microservice to satisfy goal. I choose lazy approach
                "quantity": null,
                "low_stock_threshold": null,
                "unit": null,
                "quantity_change": null
            });

            // publish - best-effort
            if let Err(e) = redis_pub.publish("product.deleted", &event).await {
                eprintln!("Redis publish error (product.deleted): {:?}", e);
            }

            // invalidate cache - best-effort
            if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
                let cache_key = format!("products:supplier:{}", supplier_id);
                let _ : Result<(), _> = conn.del(cache_key).await;
            }

            HttpResponse::Ok().body("Product deleted successfully")
        }
        Ok(_) => HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            eprintln!("DB error deleting product: {:?}", e);
            HttpResponse::InternalServerError().body("DB error")
        }
    }
}

pub async fn search_products(
    repo: web::Data<ProductRepo>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // parse optional query params
    let category = query.get("category").cloned();
    let min_price = query.get("min_price").and_then(|s| s.parse::<f64>().ok());
    let max_price = query.get("max_price").and_then(|s| s.parse::<f64>().ok());
    let supplier_id = query.get("supplier_id").and_then(|s| Uuid::parse_str(s).ok());
    let product_id = query.get("product_id").and_then(|s| Uuid::parse_str(s).ok());
    let limit = query.get("limit").and_then(|s| s.parse::<i64>().ok()).unwrap_or(50);
    let offset = query.get("offset").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);

    match repo.search_products(category, min_price, max_price, supplier_id, product_id, limit, offset).await {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            eprintln!("Search DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Search error")
        }
    }
}

pub async fn bulk_create(
    repo: web::Data<ProductRepo>,
    redis_pub: web::Data<RedisPublisher>,
    req: web::Json<BulkCreateRequest>,
) -> impl Responder {
    match repo.bulk_create(&req.products).await {
        Ok(created) => {
            // Optionally publish created events in a loop (or send a single aggregated event)
            for p in &created {
                let event = ProductEvent {
                    event_type: "product.created".to_string(),
                    product_id: p.product_id,
                    supplier_id: p.supplier_id,
                    name: Some(p.name.clone()),
                    description: p.description.clone(),
                    category: Some(p.category.clone()),
                    price: Some(p.price),
                    quantity: Some(p.quantity),
                    low_stock_threshold: None,
                    unit: Some(p.unit.clone()),
                    quantity_change: None,
                    // available: None,
                    // order_id: None,
                    // reservation_id: None,
                    // timestamp: None,
                    ..Default::default()
                };

                if let Err(e) = redis_pub.publish("product.created", &event).await {
                    eprintln!("Redis publish error in bulk: {:?}", e);
                }
            }
            HttpResponse::Created().json(created)
        }
        Err(e) => {
            eprintln!("Bulk create DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Bulk create failed")
        }
    }
}
