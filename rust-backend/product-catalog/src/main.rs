mod db;
mod handlers;
mod models;
mod rabbit_pub;
mod redis_pub;

use crate::db::ProductRepo;
use crate::redis_pub::RedisPublisher;
use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use redis::Client as RedisClient;
use sqlx::PgPool;
use std::env;

/// Starts the Product Catalog HTTP server after loading configuration, initializing logging,
/// connecting to Postgres and running migrations, and configuring Redis publisher/client
/// (falling back to a no-op publisher if Redis is unavailable).
///
/// Reads configuration from environment variables:
/// - `DATABASE_URL` (required)
/// - `REDIS_URL` (optional)
/// - `SERVICE_PORT` (defaults to `3003`)
///
/// The server registers all product- and asset-related routes and binds to `0.0.0.0:{SERVICE_PORT}`.
///
/// # Examples
///
/// ```no_run
/// // Requires a running Postgres instance and a DATABASE_URL environment variable.
/// std::env::set_var("DATABASE_URL", "postgres://user:pass@localhost/product_catalog");
/// // Optionally:
/// // std::env::set_var("REDIS_URL", "redis://127.0.0.1:6379");
/// // Start the service (runs until stopped)
/// tokio::runtime::Runtime::new().unwrap().block_on(async { main().await.unwrap() });
/// ```
///
/// # Returns
///
/// `Ok(())` if the server runs and exits without I/O errors, `Err` if binding or runtime I/O fails.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

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

    let repo = web::Data::new(ProductRepo::new(pool));
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

    println!("Product Catalog Service running on localhost:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(repo.clone())
            .app_data(redis_pub.clone())
            .app_data(redis_client.clone())
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
