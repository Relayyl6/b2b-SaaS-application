mod db;
mod handlers;
mod models;
mod rabbit_pub;
mod redis_pub;
mod storage;

use crate::db::ProductRepo;
use crate::redis_pub::RedisPublisher;
use crate::storage::{CloudinaryStorage, StorageProvider};
use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use platform::{metrics, observability};
use redis::Client as RedisClient;
use sqlx::PgPool;
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::create_product,
        handlers::get_products_for_supplier,
        handlers::get_single_product,
        handlers::update_product,
        handlers::delete_product,
        handlers::search_products,
        handlers::bulk_create,
        handlers::register_product_asset,
        handlers::list_product_assets,
        handlers::delete_product_asset,
        handlers::sign_cloudinary_upload
    ),
    components(
        schemas(
            crate::models::Product,
            crate::models::ProductAsset,
            crate::models::CreateProductRequest,
            crate::models::UpdateProductRequest,
            crate::models::BulkCreateRequest,
            crate::models::RegisterProductAssetRequest,
            crate::models::SignAssetUploadRequest,
            crate::models::SignedUploadResponse
        )
    )
)]
pub struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    observability::init_observability("product-catalog");
    metrics::init_metrics("product-catalog");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").ok();
    let port = env::var("SERVICE_PORT").unwrap_or_else(|_| "3003".into());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migrations failed");

    let db_router = web::Data::new(platform::db_router::DynamicPoolRouter::new(pool.clone()));
    let repo = web::Data::new(ProductRepo::new());
    let redis_pub = match &redis_url {
        Some(url) => match RedisPublisher::new(url).await {
            Ok(pubw) => web::Data::new(pubw),
            Err(e) => {
                eprintln!("⚠️ Failed to connect to Redis: {:?}", e);
                eprintln!("⚠️ Continuing without Redis publishing capabilities...");
                web::Data::new(RedisPublisher::new_noop())
            }
        },
        None => {
            eprintln!("⚠️ No REDIS_URL configured — using no-op publisher");
            web::Data::new(RedisPublisher::new_noop())
        }
    };

    let redis_client = web::Data::new(
        RedisClient::open(
            redis_url
                .clone()
                .unwrap_or_else(|| "redis://127.0.0.1:6379".to_string()),
        )
        .expect("redis client"),
    );

    let storage: std::sync::Arc<dyn StorageProvider> = std::sync::Arc::new(CloudinaryStorage::new(
        env::var("CLOUDINARY_CLOUD_NAME").unwrap_or_default(),
        env::var("CLOUDINARY_API_KEY").unwrap_or_default(),
        env::var("CLOUDINARY_API_SECRET").unwrap_or_default(),
    ));

    tracing::info!("Product Catalog Service listening on 0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .wrap(
                platform::middleware::tenant_middleware::TenantAuthMiddleware::with_redis(redis_client.get_ref().clone()),
            )
            .app_data(repo.clone())
            .app_data(db_router.clone())
            .app_data(redis_pub.clone())
            .app_data(redis_client.clone())
            .app_data(web::Data::new(storage.clone()))
            .route("/metrics", web::get().to(metrics::metrics_handler))
            .route("/products", web::post().to(handlers::create_product))
            .route("/products/bulk", web::post().to(handlers::bulk_create))
            .route("/products/search", web::get().to(handlers::search_products))
            .route(
                "/products/{supplier_id}/{product_id}",
                web::get().to(handlers::get_single_product),
            )
            .route(
                "/products/{supplier_id}/{product_id}",
                web::put().to(handlers::update_product),
            )
            .route(
                "/products/{supplier_id}/{product_id}",
                web::delete().to(handlers::delete_product),
            )
            .route(
                "/products/{supplier_id}/{product_id}/assets",
                web::post().to(handlers::register_product_asset),
            )
            .route(
                "/products/{supplier_id}/{product_id}/assets",
                web::get().to(handlers::list_product_assets),
            )
            .route(
                "/products/{supplier_id}/{product_id}/assets/{asset_id}",
                web::delete().to(handlers::delete_product_asset),
            )
            .route(
                "/assets/cloudinary/sign-upload",
                web::post().to(handlers::sign_cloudinary_upload),
            )
            .route(
                "/products/{supplier_id}",
                web::get().to(handlers::get_products_for_supplier),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
