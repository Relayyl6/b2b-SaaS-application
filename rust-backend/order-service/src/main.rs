use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use std::env;

mod db;
mod models;
mod routes;
mod redis_pub;

use crate::redis_pub::RedisPublisher;
use redis::Client as RedisClient;

use dotenvy::dotenv;
use std::env;
use sqlx::PgPool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    // let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let database_url = env::var("DATABASE_URL").expect("Database url must be set in the environment variable");
    let redis_url = env::var("REDIS_URL").ok();
    let port = env::var("PORT").unwrap_or_else(|_| "3004".to_string());

    let host = env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
    let addr = format!("{}:{}", host, port);

    let pool = db::get_db_pool().await;

    sqlx::migrate!("./migrations").run(&pool).await.expect("Migrations Failed");

    let redis_client = web::Data::new(RedisClient::open(redis_url.unwrap()).expect("redis client"));
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

    println!("🚀 Order Service running at http://localhost:{}", addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(redis_pub.clone())
            .app_data(redis_client.clone())
            .service(routes::create_order)
            .service(routes::get_order)
            .service(routes::update_status)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
