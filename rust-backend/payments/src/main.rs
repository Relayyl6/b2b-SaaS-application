mod db;
mod handlers;
mod models;
mod redis_sub;
mod stripe;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use platform::{metrics, observability, streams::StreamPublisher, middleware::tenant_middleware::TenantAuthMiddleware, db_router::DynamicPoolRouter};
use sqlx::postgres::PgPoolOptions;
use std::env;
use redis::Client as RedisClient;

use crate::db::PaymentRepo;

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics exported")
    ),
    security(("BearerAuth" = []), ("ApiKeyAuth" = []))
)]
pub async fn metrics_api_doc() {}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::create_payment_intent,
        handlers::get_payment_intent,
        handlers::mark_payment_succeeded,
        handlers::mark_payment_failed,
        handlers::payment_webhook,
        handlers::refund_payment_endpoint,
        handlers::transfer_payment_endpoint,
        handlers::health,
        metrics_api_doc
    ),
    components(
        schemas(
            models::CreatePaymentIntentRequest, 
            models::PaymentIntent, 
            models::PaymentWebhook, 
            models::PaymentStatus
        )
    ),
    security(
        ("BearerAuth" = []),
        ("ApiKeyAuth" = [])
    )
)]
pub struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    observability::init_observability("payments");
    metrics::init_metrics("payments");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").ok();
    let redis_raw_client = RedisClient::open(
        redis_url
            .clone()
            .unwrap_or_else(|| "redis://127.0.0.1:6379".to_string()),
    )
    .expect("redis client");
    let redis_client = web::Data::new(redis_raw_client.clone());

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

    let repo = web::Data::new(PaymentRepo::new(pool.clone()));
    let db_router = web::Data::new(DynamicPoolRouter::new(pool.clone()));
    let publisher = web::Data::new(match redis_url {
        Some(url) => StreamPublisher::new(&url).unwrap_or_else(|_| StreamPublisher::noop()),
        None => StreamPublisher::noop(),
    });

    let repo_clone = repo.clone();
    tokio::spawn(async move {
        if let Err(e) = redis_sub::listen_to_redis_events(repo_clone).await {
            tracing::error!("Payments Redis subscriber failed: {}", e);
        }
    });

    HttpServer::new(move || {
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .app_data(repo.clone())
            .app_data(db_router.clone())
            .app_data(publisher.clone())
            .app_data(redis_client.clone())
            .route("/health", web::get().to(handlers::health))
            .route("/metrics", web::get().to(metrics::metrics_handler))
            .route(
                "/payments/webhooks",
                web::post().to(handlers::payment_webhook),
            )
            .service(
                web::scope("")
                    .wrap(TenantAuthMiddleware::with_redis(redis_raw_client.clone()))
                    .route(
                        "/payments/intents",
                        web::post().to(handlers::create_payment_intent),
                    )
                    .route(
                        "/payments/intents/{id}",
                        web::get().to(handlers::get_payment_intent),
                    )
                    .route(
                        "/payments/intents/{id}/succeed",
                        web::post().to(handlers::mark_payment_succeeded),
                    )
                    .route(
                        "/payments/intents/{id}/fail",
                        web::post().to(handlers::mark_payment_failed),
                    )
                    .route(
                        "/payments/intents/{id}/refund",
                        web::post().to(handlers::refund_payment_endpoint),
                    )
                    .route(
                        "/payments/intents/{id}/transfer",
                        web::post().to(handlers::transfer_payment_endpoint),
                    ),
            )
    })
    .bind(format!("0.0.0.0:{port}"))?
    .run()
    .await
}
