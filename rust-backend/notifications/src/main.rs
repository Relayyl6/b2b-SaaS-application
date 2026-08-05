mod db;
mod dlq_pub;
mod handlers;
mod models;
mod provider;
mod redis_sub;
mod worker;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use platform::db_router::DynamicPoolRouter;
use platform::middleware::tenant_middleware::TenantAuthMiddleware;
use platform::{metrics, observability};
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::spawn;

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
        .unwrap_or_else(|_| "3010".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect postgres");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");

    let db_router = web::Data::new(DynamicPoolRouter::new(pool.clone()));
    let provider = web::Data::new(NotificationProvider::from_env());
    let dlq_publisher = web::Data::new(dlq_pub::DlqPublisher::new().await);

    let redis_client = web::Data::new(
        redis_url
            .as_ref()
            .map(|url| Client::open(url.as_str()))
            .unwrap_or_else(|| {
                eprintln!("⚠️ REDIS_URL not set — using noop client.");
                Ok(Client::open("redis://localhost:6379").unwrap())
            })
            .unwrap(),
    );

    worker::start_delivery_worker(pool.clone(), provider.clone(), dlq_publisher.clone()).await;

    if redis_url.is_some() {
        let pool_clone = pool.clone();
        spawn(async move {
            if let Err(e) = redis_sub::listen_to_redis_events(pool_clone).await {
                eprintln!("notifications redis listener stopped: {e}");
            }
        });
    }

    tracing::info!("Notifications Service listening on 0.0.0.0:{port}");

    HttpServer::new(move || {
        let tenant_middleware = TenantAuthMiddleware::with_redis(redis_client.get_ref().clone());
        App::new()
            .wrap(tenant_middleware)
            .app_data(db_router.clone())
            .app_data(provider.clone())
            .app_data(redis_client.clone())
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
