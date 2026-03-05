use actix_web::{web, App, HttpServer};

mod db;
mod models;
mod redis_pub;
mod redis_sub;
mod routes;
mod worker;
use tokio::spawn;

use crate::worker::order_expiration_worker as expiration_worker;

use crate::redis_pub::RedisPublisher;
use redis::Client as RedisClient;

use dotenvy::dotenv;
use std::env;

use crate::redis_sub::listen_to_redis_events;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let redis_url = env::var("REDIS_URL").ok();
    let port = env::var("PORT").unwrap_or_else(|_| "3006".to_string());

    let host = env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
    let addr = format!("{}:{}", host, port);

    let pool = db::get_db_pool().await;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migrations Failed");

    let redis_client = web::Data::new(
        RedisClient::open(
            redis_url
                .clone()
                .unwrap_or_else(|| "redis://127.0.0.1:6379".to_string()),
        )
        .expect("redis client"),
    );
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

    expiration_worker::start_order_expiration_worker(pool.clone(), redis_pub.get_ref().clone())
        .await;

    // spawn Redis listener in background
    let pool_clone = pool.clone();
    spawn(async move {
        let _ = listen_to_redis_events(pool_clone).await;
    });

    println!("🚀 Order Service running at http://{}", addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(redis_pub.clone())
            .app_data(redis_client.clone())
            .service(routes::create_order)
            .service(routes::get_order)
            .service(routes::update_status)
            .service(routes::delete_order)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
