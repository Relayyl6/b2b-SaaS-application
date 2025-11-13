mod models;
mod db;
mod redis_pub;
mod handlers;

use doteny::dotenv;
use std::env;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("Database url must be set in the environment variable");
    let redis_url = env::var("REDIS_URL").ok();
    let port = env::var("PORT").unwrap_or_else(|_| "3004".to_string());
}