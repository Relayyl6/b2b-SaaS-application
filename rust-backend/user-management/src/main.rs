mod models;
mod db;
mod redis_pub;
mod auth;
mod middleware;
// mod redis_sub;

mod protected;
mod unprotected;


use dotenvy::dotenv;
use std::env;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use crate::db::UserRepo;
use crate::redis_pub::RedisPublisher;
use redis::Client as RedisClient;

use crate::protected::handlers as protected_handlers;
use crate::unprotected::handlers as unprotected_handlers;

use middleware::authmiddleware::AuthMiddleware;

// use protected::handlers;

// use crate::unhandlers::{sign_up_user, sign_in_user, sign_out_user}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("Database url must be set in the environment variable");
    let redis_url = env::var("REDIS_URL").ok();
    let port = env::var("PORT").unwrap_or_else(|_| "3004".to_string());
    let jwt_secret = env::var("SECRET").unwrap_or_else(|_| "something".to_string());

    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to postgres database");
    sqlx::migrate!("./migrations").run(&pool).await.expect("Migrations Failed");

    let repo = web::Data::new(UserRepo::new(pool.clone()));

    let middleware = AuthMiddleware::new(pool.clone(), jwt_secret.clone());

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
                web::scope("/protected")       // all /protected/* routes
                    .wrap(middleware.clone())  // middleware only applies here
                    .route("/update/{id}", web::put().to(protected_handlers::update_user_handler))
                    .route("/delete/{id}", web::delete().to(protected_handlers::delete_user_handler))
            )
            // other unprotected routes outside the scope
            .route("/signup", web::post().to(unprotected_handlers::sign_up_user))
            .route("/signin", web::post().to(unprotected_handlers::sign_in_user))
            .route("/signout", web::post().to(unprotected_handlers::sign_out_user))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}