mod db;
mod handlers;
mod models;
mod publisher;
mod redis_sub;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::spawn;

use crate::publisher::RedisPublisher;
use crate::redis_sub::listen_to_redis_events;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let redis_url = env::var("REDIS_URL");
    let port = env::var("SERVICE_PORT").unwrap_or_else(|_| "3006".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("failed to connect postgres");

    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Migration failed: {e:?}");
        std::process::exit(1);
    }

    let repo = web::Data::new(db::LogisticsRepo::new(&pool));

    let redis_client = web::Data::new(
        redis_url
            .as_ref()
            .map(|url| Client::open(url.as_str()))
            .unwrap_or_else(|_| Ok(Client::open("redis://localhost:6379").expect("redis fallback")))
            .expect("redis client"),
    );

    let redis_pub = match redis_url.clone() {
        Ok(url) => {
            match RedisPublisher::new(&url).await {
                Ok(p) => web::Data::new(p),
                Err(e) => {
                    eprintln!("Failed to connect redis for publishing: {e:?}. continuing with noop publisher");
                    web::Data::new(RedisPublisher::new_noop())
                }
            }
        }
        Err(_) => web::Data::new(RedisPublisher::new_noop()),
    };

    let repo_clone = repo.clone();
    let redis_pub_clone = redis_pub.clone();
    if redis_url.is_ok() {
        spawn(async move {
            if let Err(e) = listen_to_redis_events(repo_clone, redis_pub_clone).await {
                eprintln!("redis listener stopped: {e}");
            }
        });
    }

    HttpServer::new(move || {
        App::new()
            .app_data(repo.clone())
            .app_data(redis_pub.clone())
            .app_data(redis_client.clone())
            .route("/shipments", web::post().to(handlers::create_shipment))
            .route(
                "/shipments/{shipment_id}",
                web::get().to(handlers::get_shipment),
            )
            .route(
                "/shipments/supplier/{supplier_id}",
                web::get().to(handlers::list_supplier_shipments),
            )
            .route(
                "/shipments/{shipment_id}/status",
                web::put().to(handlers::update_status),
            )
    })
    .bind(format!("0.0.0.0:{port}"))?
    .run()
    .await
}
