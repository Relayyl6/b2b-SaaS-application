mod db;
mod handlers;
mod models;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use platform::{metrics, observability, streams::StreamPublisher};
use sqlx::postgres::PgPoolOptions;
use std::env;

use crate::db::SupplierRepo;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    observability::init_observability("supplier-management");
    metrics::init_metrics("supplier-management");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").ok();
    let port = env::var("SERVICE_PORT")
        .or_else(|_| env::var("PORT"))
        .unwrap_or_else(|_| "3011".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");

    let repo = web::Data::new(SupplierRepo::new(pool));
    let publisher = web::Data::new(match redis_url {
        Some(url) => StreamPublisher::new(&url).unwrap_or_else(|_| StreamPublisher::noop()),
        None => StreamPublisher::noop(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(repo.clone())
            .app_data(publisher.clone())
            .route("/health", web::get().to(handlers::health))
            .route("/metrics", web::get().to(metrics::metrics_handler))
            .route("/suppliers", web::post().to(handlers::create_supplier))
            .route("/suppliers/{id}", web::get().to(handlers::get_supplier))
            .route(
                "/suppliers/owner/{owner_user_id}",
                web::get().to(handlers::list_owner_suppliers),
            )
            .route(
                "/suppliers/{id}/status",
                web::put().to(handlers::update_supplier_status),
            )
    })
    .bind(format!("0.0.0.0:{port}"))?
    .run()
    .await
}
