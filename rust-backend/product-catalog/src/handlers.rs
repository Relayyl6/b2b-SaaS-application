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

/// Creates a product and emits best-effort integration events.
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
            redis_pub.publish_async("product.created", event.clone());
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

/// Returns all products for a supplier and emits view events.
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
                redis_pub.publish_async("product.viewed", event.clone());
            }
            HttpResponse::Ok().json(&items)
        }
        Err(e) => {
            eprintln!("Get products DB error: {:?}", e);
            HttpResponse::InternalServerError().body("DB error")
        }
    }
}

/// Returns a single product by supplier and product id.
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

/// Updates a product and emits a product.updated event.
pub async fn update_product(
    repo: web::Data<ProductRepo>,
    redis_pub: web::Data<RedisPublisher>,
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<UpdateProductRequest>,
) -> impl Responder {
    let (supplier_id, product_id) = path.into_inner();
    let mut update_data = req.into_inner();

    if update_data.quantity.is_some() && update_data.quantity_change.is_some() {
        return HttpResponse::BadRequest()
            .body("Provide either quantity or quantity_change, not both");
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
            redis_pub.publish_async("product.updated", event.clone());

            HttpResponse::Ok().json(p)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            eprintln!("DB error updating product: {:?}", e);
            HttpResponse::InternalServerError().body("DB error")
        }
    }
}

/// Deletes a product, emits product.deleted, and invalidates cache.
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
            redis_pub.publish_async("product.deleted", event.clone());

            let cache_key = format!("products:supplier:{}", supplier_id);
            match redis_client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    if let Err(e) = conn.del::<_, ()>(&cache_key).await {
                        eprintln!(
                            "Redis cache invalidation error (delete_product): supplier_id={}, cache_key={}, err={:?}",
                            supplier_id, cache_key, e
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Redis connection error for cache invalidation (delete_product): supplier_id={}, cache_key={}, err={:?}",
                        supplier_id, cache_key, e
                    );
                }
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

/// Searches products by optional query parameters.
pub async fn search_products(
    repo: web::Data<ProductRepo>,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let category = query.get("category").cloned();

    let min_price = match query.get("min_price") {
        Some(v) => match v.parse::<f64>() {
            Ok(parsed) => Some(parsed),
            Err(_) => return HttpResponse::BadRequest().body("Invalid min_price"),
        },
        None => None,
    };
    let max_price = match query.get("max_price") {
        Some(v) => match v.parse::<f64>() {
            Ok(parsed) => Some(parsed),
            Err(_) => return HttpResponse::BadRequest().body("Invalid max_price"),
        },
        None => None,
    };

    let supplier_id = match query.get("supplier_id") {
        Some(v) => match Uuid::parse_str(v) {
            Ok(id) => Some(id),
            Err(_) => return HttpResponse::BadRequest().body("Invalid supplier_id"),
        },
        None => None,
    };
    let product_id = match query.get("product_id") {
        Some(v) => match Uuid::parse_str(v) {
            Ok(id) => Some(id),
            Err(_) => return HttpResponse::BadRequest().body("Invalid product_id"),
        },
        None => None,
    };

    let limit = match query.get("limit") {
        Some(v) => match v.parse::<i64>() {
            Ok(parsed) => parsed,
            Err(_) => return HttpResponse::BadRequest().body("Invalid limit"),
        },
        None => 50,
    }
    .clamp(1, 200);
    let offset = match query.get("offset") {
        Some(v) => match v.parse::<i64>() {
            Ok(parsed) => parsed,
            Err(_) => return HttpResponse::BadRequest().body("Invalid offset"),
        },
        None => 0,
    }
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

/// Creates products in bulk and emits product.created events.
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
                redis_pub.publish_async("product.created", event.clone());
            }
            HttpResponse::Created().json(created)
        }
        Err(e) => {
            eprintln!("Bulk create DB error: {:?}", e);
            HttpResponse::InternalServerError().body("Bulk create failed")
        }
    }
}

/// Stores uploaded asset metadata for a product.
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

/// Lists stored asset metadata for a product.
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

/// Deletes product asset metadata by asset id.
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

/// Generates signed Cloudinary upload parameters for direct client uploads.
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
    if !folder.starts_with("b2b-saas/products") || folder.contains("..") {
        return HttpResponse::BadRequest().body("Invalid folder");
    }
    if let Some(public_id) = &req.public_id {
        let valid = public_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '/');
        if !valid || public_id.contains("..") {
            return HttpResponse::BadRequest().body("Invalid public_id");
        }
    }
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
