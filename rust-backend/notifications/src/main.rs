mod db;
mod handlers;
mod models;
mod provider;
mod redis_sub;
mod worker;
mod dlq_pub;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use platform::{metrics, observability};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::spawn;

use crate::db::NotificationRepo;
use crate::provider::NotificationProvider;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    observability::init_observability("notifications");
    metrics::init_metrics("notifications");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").ok();
    let port = env::var("SERVICE_PORT")
        .or_else(|_| env::var("PORT"))
        .unwrap_or_else(|_| "3009".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect postgres");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");

    let repo = web::Data::new(NotificationRepo::new(pool));
    let provider = web::Data::new(NotificationProvider::from_env());
    let dlq_publisher = web::Data::new(dlq_pub::DlqPublisher::new().await);

    worker::start_delivery_worker(repo.clone(), provider.clone(), dlq_publisher.clone()).await;

    if redis_url.is_some() {
        let repo_clone = repo.clone();
        spawn(async move {
            if let Err(e) = redis_sub::listen_to_redis_events(repo_clone).await {
                eprintln!("notifications redis listener stopped: {e}");
            }
        });
    }

    tracing::info!("Notifications Service listening on 0.0.0.0:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(repo.clone())
            .app_data(provider.clone())
            .route("/health", web::get().to(handlers::health))
            .route("/metrics", web::get().to(metrics::metrics_handler))
            .route(
                "/notifications",
                web::post().to(handlers::create_notification),
            )
            .route(
                "/notifications",
                web::get().to(handlers::list_notifications),
            )
            .route(
                "/notifications/{id}",
                web::get().to(handlers::get_notification),
            )
            .route(
                "/notifications/{id}/read",
                web::put().to(handlers::mark_notification_read),
            )
            .route(
                "/notification-devices",
                web::post().to(handlers::register_device),
            )
            .route(
                "/notification-devices/user/{user_id}",
                web::get().to(handlers::list_user_devices),
            )
            .route(
                "/notification-devices/{id}",
                web::delete().to(handlers::disable_device),
            )
            .route(
                "/notification-preferences/user/{user_id}",
                web::get().to(handlers::get_preferences),
            )
            .route(
                "/notification-preferences/user/{user_id}",
                web::put().to(handlers::update_preferences),
            )
    })
    .bind(format!("0.0.0.0:{port}"))?
    .run()
    .await
}
