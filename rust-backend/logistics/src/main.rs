mod db;
mod handlers;
mod models;
mod publisher;
mod rabbit_pub;
mod redis_sub;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use platform::db_router::DynamicPoolRouter;
use platform::middleware::tenant_middleware::TenantAuthMiddleware;
use platform::{metrics, observability};
use redis::Client as RedisClient;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::spawn;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::publisher::RedisPublisher;
use crate::rabbit_pub::RabbitPublisher;
use crate::redis_sub::listen_to_redis_events;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::create_shipment,
        handlers::get_shipment,
        handlers::list_supplier_shipments,
        handlers::update_status,
        handlers::cancel_shipment_by_order,
        handlers::health
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    observability::init_observability("logistics");
    metrics::init_metrics("logistics");

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let redis_url = env::var("REDIS_URL");
    let port = env::var("SERVICE_PORT").unwrap_or_else(|_| "3008".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("failed to connect postgres");

    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Migration failed: {e:?}");
        std::process::exit(1);
    }

    let db_router = web::Data::new(DynamicPoolRouter::new(pool.clone()));
    let repo = web::Data::new(db::LogisticsRepo::new());

    let raw_redis_client = redis_url
        .as_ref()
        .map(|url| RedisClient::open(url.as_str()))
        .unwrap_or_else(|_| Ok(RedisClient::open("redis://localhost:6379").expect("redis fallback")))
        .expect("redis client");
    let redis_client = web::Data::new(raw_redis_client.clone());

    let amqp_addr = env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into());
    let rabbit_pub = match RabbitPublisher::new(&amqp_addr).await {
        Ok(p) => web::Data::new(p),
        Err(e) => {
            eprintln!("Failed to connect RabbitMQ: {e:?}");
            std::process::exit(1);
        }
    };

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

    let db_router_clone = db_router.clone();
    let repo_clone = repo.clone();
    let redis_pub_clone = redis_pub.clone();
    let rabbit_pub_clone = rabbit_pub.clone();
    if redis_url.is_ok() {
        spawn(async move {
            if let Err(e) =
                listen_to_redis_events(db_router_clone, repo_clone, redis_pub_clone, rabbit_pub_clone).await
            {
                eprintln!("redis listener stopped: {e}");
            }
        });
    }

    HttpServer::new(move || {
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .wrap(TenantAuthMiddleware::with_redis(raw_redis_client.clone()))
            .app_data(db_router.clone())
            .app_data(repo.clone())
            .app_data(redis_pub.clone())
            .app_data(rabbit_pub.clone())
            .app_data(redis_client.clone())
            .route("/health", web::get().to(handlers::health))
            .route("/metrics", web::get().to(metrics::metrics_handler))
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
            .route(
                "/shipments/order/{order_id}/cancel",
                web::put().to(handlers::cancel_shipment_by_order),
            )
    })
    .bind(format!("0.0.0.0:{port}"))?
    .run()
    .await
}
