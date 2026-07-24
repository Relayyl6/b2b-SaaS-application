
// src/main.rs
mod db;
mod handlers;
mod models;
mod redis_pub;
mod redis_sub;
mod worker;

use crate::redis_pub::RedisPublisher;
use crate::redis_sub::listen_to_redis_events;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use platform::{metrics, observability};
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::spawn;

use crate::worker::reservation_worker;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    observability::init_observability("inventory-management");
    metrics::init_metrics("inventory-management");

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let redis_url = env::var("REDIS_URL");
    let port = env::var("SERVICE_PORT").unwrap_or_else(|_| "3006".into());

    // println!("Connecting to DB: {}", db_url);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("postgres");
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("❌ Migration failed: {:?}", e);
        std::process::exit(1);
    }

    let repo = web::Data::new(db::InventoryRepo::new(&pool));

    // let redis_client = web::Data::new(Client::open(redis_url.clone()));

    let redis_client = web::Data::new(
        redis_url
            .as_ref()
            .map(|url| Client::open(url.as_str()))
            .unwrap_or_else(|_| {
                eprintln!("⚠️ REDIS_URL not set — using noop client.");
                Ok(Client::open("redis://localhost:6379").unwrap())
            })
            .unwrap(),
    );

    let redis_pub = match redis_url.clone() {
        Ok(ref url) => match RedisPublisher::new(url).await {
            Ok(pubw) => web::Data::new(pubw),
            Err(e) => {
                eprintln!("⚠️ Failed to connect to Redis: {:?} ⚠️ Continuing without Redis publishing capabilities...", e);
                web::Data::new(RedisPublisher::new_noop())
            }
        },
        Err(e) => {
            eprintln!(
                "⚠️ No REDIS_URL configured — using no-op publisher: {:?}",
                e
            );
            web::Data::new(RedisPublisher::new_noop())
        }
    };

    reservation_worker::start_reservation_expiration_worker(pool.clone(), redis_pub.clone()).await;

    // spawn Redis listener in background
    let pool_clone = pool.clone();
    let repo_clone = repo.clone();
    let redis_pub_clone = redis_pub.clone();

    spawn(async move {
        let _ = listen_to_redis_events(pool_clone, repo_clone, redis_pub_clone).await;
    });

    tracing::info!("Inventory Service listening on 0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(repo.clone())
            .app_data(redis_pub.clone())
            .app_data(redis_client.clone())
            .route("/metrics", web::get().to(metrics::metrics_handler))
            .route("/inventory", web::post().to(handlers::create_inventory))
            .route(
                "/inventory/{supplier_id}/{product_id}",
                web::get().to(handlers::get_inventory_item),
            )
            .route(
                "/inventory/{supplier_id}",
                web::get().to(handlers::get_inventory),
            )
            .route(
                "/inventory/{supplier_id}/update",
                web::post().to(handlers::update_stock),
            )
            .route(
                "/inventory/{supplier_id}/{product_id}",
                web::delete().to(handlers::delete_product),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
