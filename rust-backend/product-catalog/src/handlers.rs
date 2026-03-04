use crate::db::ProductRepo;
use crate::models::{
    BulkCreateRequest, CreateProductRequest, ProductEvent, RegisterProductAssetRequest,
    SignAssetUploadRequest, SignedUploadResponse, UpdateProductRequest,
};
use crate::rabbit_pub::publish_example_event;
use crate::redis_pub::RedisPublisher;
use actix_web::{HttpResponse, Responder, web};
use chrono::Utc;
use redis::AsyncCommands;
use serde_json::json;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

pub async fn create_product(
    repo: web::Data<ProductRepo>,
    redis_pub: web::Data<RedisPublisher>,
    req: web::Json<CreateProductRequest>,
) -> impl Responder {
    match repo.create_product(&req).await {
        Ok(product) => {
            let event = ProductEvent {
                event_type: "product.created".to_string(),
                product_id: product.product_id,
                supplier_id: product.supplier_id,
                price: Some(product.price),
                category: Some(product.category.clone()),
                name: Some(product.name.clone()),
                description: product.description.clone(),
                quantity: Some(product.quantity),
                low_stock_threshold: Some(product.low_stock_threshold),
                unit: Some(product.unit.clone()),
                quantity_change: None,
                ..Default::default()
            };

            if let Err(e) = redis_pub.publish("product.created", &event).await {
                eprintln!("Redis publish error (product.created): {:?}", e);
            }
            if let Err(e) = publish_example_event(&event).await {
                eprintln!("Rabbit publish error (product.created): {:?}", e);
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
    redis_pub: web::Data<RedisPublisher>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let supplier_id = path.into_inner();
    match repo.get_by_supplier(supplier_id).await {
        Ok(items) => {
            for item in &items {
                let event = ProductEvent {
                    event_type: "product.viewed".to_string(),
                    product_id: item.product_id,
                    supplier_id: item.supplier_id,
                    price: Some(item.price),
                    category: Some(item.category.clone()),
                    name: Some(item.name.clone()),
                    description: item.description.clone(),
                    quantity: Some(item.quantity),
                    low_stock_threshold: Some(item.low_stock_threshold),
                    unit: Some(item.unit.clone()),
                    quantity_change: None,
                    ..Default::default()
                };

                if let Err(e) = redis_pub.publish("product.viewed", &event).await {
                    eprintln!("Redis publish error (product.viewed): {:?}", e);
                }
            }
            HttpResponse::Ok().json(&items)
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
    let mut update_data = req.into_inner();

    if update_data.quantity_change.is_some() {
        update_data.quantity = None;
    }

    match repo
        .update_product(supplier_id, product_id, &update_data)
        .await
    {
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
            });

            if let Err(e) = redis_pub.publish("product.deleted", &event).await {
                eprintln!("Redis publish error (product.deleted): {:?}", e);
            }

            if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
                let cache_key = format!("products:supplier:{}", supplier_id);
                let _: Result<(), _> = conn.del(cache_key).await;
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
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let category = query.get("category").cloned();
    let min_price = query.get("min_price").and_then(|s| s.parse::<f64>().ok());
    let max_price = query.get("max_price").and_then(|s| s.parse::<f64>().ok());
    let supplier_id = query
        .get("supplier_id")
        .and_then(|s| Uuid::parse_str(s).ok());
    let product_id = query
        .get("product_id")
        .and_then(|s| Uuid::parse_str(s).ok());
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(50)
        .clamp(1, 200);
    let offset = query
        .get("offset")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0)
        .max(0);

    match repo
        .search_products(
            category,
            min_price,
            max_price,
            product_id,
            supplier_id,
            limit,
            offset,
        )
        .await
    {
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
                    low_stock_threshold: Some(p.low_stock_threshold),
                    unit: Some(p.unit.clone()),
                    quantity_change: None,
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

pub async fn register_product_asset(
    repo: web::Data<ProductRepo>,
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<RegisterProductAssetRequest>,
) -> impl Responder {
    let (supplier_id, product_id) = path.into_inner();
    match repo
        .register_product_asset(supplier_id, product_id, &req)
        .await
    {
        Ok(asset) => HttpResponse::Created().json(asset),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Product not found"),
        Err(e) => {
            eprintln!("Register product asset DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to register product asset")
        }
    }
}

pub async fn list_product_assets(
    repo: web::Data<ProductRepo>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (supplier_id, product_id) = path.into_inner();
    match repo.list_product_assets(supplier_id, product_id).await {
        Ok(assets) => HttpResponse::Ok().json(assets),
        Err(e) => {
            eprintln!("List product assets DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to list product assets")
        }
    }
}

pub async fn delete_product_asset(
    repo: web::Data<ProductRepo>,
    path: web::Path<(Uuid, Uuid, Uuid)>,
) -> impl Responder {
    let (supplier_id, product_id, asset_id) = path.into_inner();
    match repo
        .delete_product_asset(supplier_id, product_id, asset_id)
        .await
    {
        Ok(0) => HttpResponse::NotFound().body("Asset not found"),
        Ok(_) => HttpResponse::Ok().body("Asset deleted"),
        Err(e) => {
            eprintln!("Delete product asset DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to delete product asset")
        }
    }
}

pub async fn sign_cloudinary_upload(req: web::Json<SignAssetUploadRequest>) -> impl Responder {
    let cloud_name = match env::var("CLOUDINARY_CLOUD_NAME") {
        Ok(v) => v,
        Err(_) => return HttpResponse::ServiceUnavailable().body("Missing CLOUDINARY_CLOUD_NAME"),
    };
    let api_key = match env::var("CLOUDINARY_API_KEY") {
        Ok(v) => v,
        Err(_) => return HttpResponse::ServiceUnavailable().body("Missing CLOUDINARY_API_KEY"),
    };
    let api_secret = match env::var("CLOUDINARY_API_SECRET") {
        Ok(v) => v,
        Err(_) => return HttpResponse::ServiceUnavailable().body("Missing CLOUDINARY_API_SECRET"),
    };

    let folder = req
        .folder
        .clone()
        .unwrap_or_else(|| "b2b-saas/products".to_string());
    let timestamp = Utc::now().timestamp();

    let mut sign_parts = vec![format!("folder={folder}"), format!("timestamp={timestamp}")];
    if let Some(public_id) = &req.public_id {
        sign_parts.push(format!("public_id={public_id}"));
    }
    sign_parts.sort();
    let to_sign = format!("{}{}", sign_parts.join("&"), api_secret);
    let mut hasher = Sha1::new();
    hasher.update(to_sign.as_bytes());
    let signature = format!("{:x}", hasher.finalize());

    HttpResponse::Ok().json(SignedUploadResponse {
        cloud_name,
        api_key,
        timestamp,
        signature,
        folder,
        public_id: req.public_id.clone(),
    })
}
