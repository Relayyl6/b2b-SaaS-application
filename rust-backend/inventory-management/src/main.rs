// src/main.rs
mod models;
mod db;
mod redis_pub;
mod handlers;
mod redis_sub;
mod worker;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use std::env;
use tokio::spawn;
use sqlx::postgres::PgPoolOptions;
use redis::Client;
use crate::redis_sub::listen_to_redis_events;

use crate::worker::reservation_worker as reservation_worker;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL");
    let port = env::var("SERVICE_PORT").unwrap_or_else(|_| "3002".into());

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
    let redis_client = web::Data::new(Client::open(redis_url).unwrap());

    reservation_worker::start_reservation_expiration_worker(pool.clone(), redis_pub.clone()).await;

    // let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let _redis_conn = redis_client.get_async_connection().await.expect("Redis connection failed");

    // spawn Redis listener in background
    let pool_clone = pool.clone();
    spawn(async move {
        let _ = listen_to_redis_events(pool_clone).await;
    });

    println!("Inventory Service running on http://localhost:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(repo.clone())
            .app_data(redis_pub.clone())
            .app_data(redis_client.clone())
            .route("/inventory", web::post().to(handlers::create_inventory))
            .route(
                "/inventory/{supplier_id}/{product_id}",
                web::get().to(handlers::get_inventory_item),
            )
            .route("/inventory/{supplier_id}", web::get().to(handlers::get_inventory))
            .route("/inventory/{supplier_id}/update", web::post().to(handlers::update_stock))
            .route(
                "/inventory/{supplier_id}/{product_id}",
                web::delete().to(handlers::delete_product),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}