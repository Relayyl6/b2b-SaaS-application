mod models;
mod db;
mod redis_pub;
mod handlers;
mod auth;
mod authmiddleware;
mod redis_pub;
mod redis_sub;

use doteny::dotenv;
use std::env;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use crate::db::ProductRepo;
use crate::redis_pub::RedisPublisher;
use redis::Client as RedisClient;

use crate::db::UserRepo;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("Database url must be set in the environment variable");
    let redis_url = env::var("REDIS_URL").ok();
    let port = env::var("PORT").unwrap_or_else(|_| "3004".to_string());

    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to postgres database");
    sqlx::migrate("./migrations").run(&pool).await.expect("Migrations Failed");

    let repo = web::Data::new(UserRepo::new(pool));

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

    let redis_client = web::Data::new(RedisClient::open(redis_url.unwrap()).expect("redis client"));

    println!("User management Service running on localhost:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(repo.clone())
            .app_data(redis_pub.clone())
            .app_data(redis_client.clone())
            .service(
                web::scope("/protected")       // all /products/* routes
                    .wrap(authmiddleware.clone())  // middleware only applies here
                    .route("/update/{id}", web::put().to(handlers::update_user_handler))
                    .route("/delete/{id}", web::delete().to(handlers::delete_user_handler))
            )
            // other unprotected routes outside the scope
            .route("/products", web::post().to(handlers::create_product))
            .route("/products/bulk", web::post().to(handlers::bulk_create))
            .route("/products/search", web::get().to(handlers::search_products))
            .route("/products/{supplier_id}/{product_id}", web::get().to(handlers::get_single_product))
            .route("/products/{supplier_id}/{product_id}", web::put().to(handlers::update_product))
            .route("/products/{supplier_id}/{product_id}", web::delete().to(handlers::delete_product))
            .route("/products/{supplier_id}", web::get().to(handlers::get_products_for_supplier))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}