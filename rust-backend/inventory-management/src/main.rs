// src/main.rs
mod models;
mod db;
mod redis_pub;
mod handlers;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use std::env;
use sqlx::PgPool;
use redis::Client as RedisClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL");
    let port = env::var("SERVICE_PORT").unwrap_or_else(|_| "3002".into());

    // println!("Connecting to DB: {}", db_url);
    let pool = PgPool::connect(&db_url).await.expect("postgres");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let repo = web::Data::new(db::InventoryRepo::new(pool));
    let redis_pub = web::Data::new(redis_pub::RedisPublisher::new(&redis_url));
    let redis_client = web::Data::new(RedisClient::open(redis_url).unwrap());

    println!("Inventory Service running on :{}", port);

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
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}