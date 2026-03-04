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

/// Creates a new product and publishes best-effort "product.created" integration events.
///
/// On success responds with HTTP 201 Created containing the newly created product JSON. If the repository
/// operation fails, responds with HTTP 500 Internal Server Error.
///
/// # Examples
///
/// ```no_run
/// use actix_web::{test, web, App, http::StatusCode};
/// use uuid::Uuid;
/// use product_catalog::{handlers::create_product, repos::ProductRepo, models::CreateProductRequest};
///
/// # async fn example() {
/// // Set up test app with a ProductRepo and Redis publisher wired in (omitted).
/// // Build a CreateProductRequest and call the handler, expecting Created.
/// let req = CreateProductRequest {
///     supplier_id: Uuid::new_v4(),
///     name: "Example".into(),
///     description: None,
///     price: 100,
///     category: "tools".into(),
///     quantity: 10,
///     low_stock_threshold: 1,
///     unit: "pcs".into(),
///     ..Default::default()
/// };
///
/// // Sending the request to the handler should yield 201 Created on success.
/// // let resp = test::call_service(&app, TestRequest::post().set_json(&req).to_request()).await;
/// // assert_eq!(resp.status(), StatusCode::CREATED);
/// # }
/// ```
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

/// Fetches all products for the given supplier and emits a `product.viewed` event for each item.
///
/// On success returns the list of products belonging to the supplier. For every returned product a
/// `product.viewed` event is published to the configured Redis publisher; publish failures are
/// logged but do not affect the HTTP response.
///
/// # Examples
///
/// ```no_run
/// use actix_web::web;
/// use uuid::Uuid;
///
/// // assuming `repo` and `redis_pub` are previously constructed `web::Data` instances
/// let supplier_id = Uuid::new_v4();
/// // call the handler (within an async context / test runtime)
/// // let resp = get_products_for_supplier(repo, redis_pub, web::Path::from(supplier_id)).await;
/// ```
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

/// Fetches a product by supplier and product ID and returns an HTTP response with the result.
///
/// On success returns `200 OK` with the product serialized as JSON. Returns `404 Not Found` if the
/// product does not exist, and `500 Internal Server Error` on database errors.
///
/// # Examples
///
/// ```no_run
/// use actix_web::web;
/// use uuid::Uuid;
///
/// // Given an initialized `repo: web::Data<ProductRepo>` and valid UUIDs:
/// let path = web::Path::from((Uuid::new_v4(), Uuid::new_v4()));
/// let resp = get_single_product(repo, path).await;
/// ```
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

/// Updates the specified product and emits a `product.updated` event for downstream consumers.
///
/// On success returns the updated product as the HTTP response body. If the product is not found,
/// responds with 404 Not Found. Database errors produce a 500 Internal Server Error.
///
/// # Examples
///
/// ```no_run
/// use actix_web::{test, web, App};
/// use uuid::Uuid;
///
/// // Build an HTTP request to the handler (details such as repo and redis publisher setup are omitted).
/// // This example demonstrates the request shape; handler wiring is environment-specific.
/// let supplier_id = Uuid::new_v4();
/// let product_id = Uuid::new_v4();
/// let req_body = serde_json::json!({
///     "name": "Updated name",
///     "price": 19.99,
///     "quantity_change": 5
/// });
///
/// // Example test flow (pseudo):
/// // let app = test::init_service(App::new().route(...)).await;
/// // let req = test::TestRequest::put()
/// //     .uri(&format!("/suppliers/{}/products/{}", supplier_id, product_id))
/// //     .set_json(&req_body)
/// //     .to_request();
/// // let resp = test::call_service(&app, req).await;
/// // assert!(resp.status().is_success());
/// ```
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

/// Searches for products using optional query parameters.
///
/// Supported query parameters:
/// - `category`: product category to filter by.
/// - `min_price`, `max_price`: price range (parsed as floating point).
/// - `supplier_id`, `product_id`: UUIDs to filter by.
/// - `limit`: number of results to return (defaults to 50, clamped to 1..=200).
/// - `offset`: result offset (defaults to 0, minimum 0).
///
/// Returns an HTTP 200 response with the matching product rows on success,
/// or an HTTP 500 response with body "Search error" if the repository call fails.
///
/// # Examples
///
/// ```no_run
/// use actix_web::web;
/// use std::collections::HashMap;
///
/// // Build query parameters as they would come from a request.
/// let mut params = HashMap::new();
/// params.insert("category".to_string(), "tools".to_string());
/// params.insert("limit".to_string(), "10".to_string());
///
/// let query: web::Query<HashMap<String, String>> = web::Query::from(params);
///
/// // `repo` would be provided by the Actix application state in real usage.
/// // let response = search_products(repo, query).await;
/// ```
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

/// Get metadata for all assets attached to the specified product.
///
/// # Examples
///
/// ```ignore
/// use actix_web::web;
/// use uuid::Uuid;
///
/// // `repo` is a `web::Data<ProductRepo>` prepared in test setup.
/// let supplier_id = Uuid::new_v4();
/// let product_id = Uuid::new_v4();
/// let response = list_product_assets(repo, web::Path::from((supplier_id, product_id))).await;
/// ```
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

/// Deletes the metadata record for a product asset identified by supplier, product, and asset IDs.
///
/// Returns HTTP 200 OK with "Asset deleted" when a row was removed, HTTP 404 Not Found with "Asset not found" when there was no matching record, and HTTP 500 Internal Server Error if the repository operation fails.
///
/// # Examples
///
/// ```no_run
/// use actix_web::web;
/// use uuid::Uuid;
///
/// // Assume `repo` is a `web::Data<ProductRepo>` already initialized.
/// let supplier_id = Uuid::new_v4();
/// let product_id = Uuid::new_v4();
/// let asset_id = Uuid::new_v4();
/// let path = web::Path::from((supplier_id, product_id, asset_id));
///
/// // Call the handler in an async context:
/// // let resp = delete_product_asset(repo.clone(), path).await;
/// ```
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
///
/// Reads Cloudinary credentials from the environment and returns a JSON payload containing
/// the `cloud_name`, `api_key`, `timestamp`, `signature`, `folder`, and optional `public_id`.
/// If any required environment variable is missing, responds with HTTP 503 and a short error message.
///
/// # Examples
///
/// ```
/// // Construct a request and call the handler (async context required).
/// let req = SignAssetUploadRequest { folder: None, public_id: None };
/// let resp = actix_rt::System::new().block_on(async { sign_cloudinary_upload(web::Json(req)).await });
/// // `resp` is an HTTP response whose JSON body is `SignedUploadResponse`.
/// ```
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
