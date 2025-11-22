use sqlx::{Pool, Postgres};
use std::env;
use dotenvy::dotenv;

pub async fn get_db_pool() -> Pool<Postgres> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres")
}
