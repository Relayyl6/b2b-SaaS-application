use actix_web::{App, HttpServer, web};
use redis::AsyncCommands;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use redis::Client as RedisClient;
use events::{create_product_from_event, update_product_from_event, delete_product_from_event}

mod handlers;
use handlers::*;

pub async fn listen_to_redis_events(pool: PgPool) {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL");
    let redis_client = web::Data::new(RedisClient::open(redis_url).unwrap());
    let mut redis_conn = redis_client.get_async_connection().await.unwrap();
    let mut pubsub = redis_conn.as_pubsub();

    pubsub.subscribe("product.events").await.unwrap();
    println!("📡 Subscribed to product.events");

    while let Ok(msg) = pubsub.on_message().await {
        if let Ok(payload): Result<String, _> = msg.get_payload() {
            if let Ok(event): Result<ProductEvent, _> = serde_json::from_str(&payload) {
                match event.event_type.as_str() {
                    "product.created" => {
                        if let Err(e) = create_product_from_event(&pool, event).await {
                            eprintln!("Error handling product.created: {:?}", e);
                        }
                    }
                    "product.updated" => {
                        if let Err(e) = update_product_from_event(&pool, event).await {
                            eprintln!("Error handling product.updated: {:?}", e);
                        }
                    }
                    "product.deleted" => {
                        if let Err(e) = delete_product_from_event(&pool, event).await {
                            eprintln!("Error handling product.deleted: {:?}", e);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}