mod events;
mod handlers;
mod models;
mod publisher;
mod tests;
mod worker;

use crate::handlers::AnalyticsRepo;
use crate::worker::consumer::RabbitConsumer;
use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use platform::middleware::tenant_middleware::TenantAuthMiddleware;
use platform::db_router::DynamicPoolRouter;
use tokio::spawn;
use tracing::{error, subscriber};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::get_analytics,
        handlers::post_analytics,
        handlers::health
    ),
    components(
        schemas(
            models::AnalyticsRequestBody,
            models::AnalyticsRequestQuery
        )
    )
)]
struct ApiDoc;
use tracing_subscriber::FmtSubscriber;

// The analytics service might consume events like:
//          InventoryViewed
//          ProductClicked
//          OrderInitiated
//          OrderCompleted
//          PaymentProcessed

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // tracing
    let subscriber = FmtSubscriber::builder().with_env_filter("info").finish();
    let _ = subscriber::set_global_default(subscriber);

    let port = env::var("SERVICE_PORT").unwrap_or_else(|_| "3007".to_string());
    let db_url = env::var("DATABASE_URL").expect("Database url not set");
    let redis_url = env::var("REDIS_URL");

    // Redis client
    let redis_client = web::Data::new(
        redis_url
            .as_ref()
            .map(|url| Client::open(url.as_str()))
            .unwrap_or_else(|_| {
                eprintln!("⚠️ REDIS_URL not set — using noop client.");
                Ok(Client::open("redis://localhost:6379").unwrap())
            })
            .unwrap(),
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("postgres");
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("❌ Migration failed: {:?}", e);
        std::process::exit(1);
    };

    let repo = web::Data::new(AnalyticsRepo::new(&pool));
    let db_router = web::Data::new(DynamicPoolRouter::new(pool.clone()));

    let rabbitconsume = web::Data::new(RabbitConsumer::new(&pool));
    let consumer = rabbitconsume.clone();

    let pool_clone = pool.clone();
    let redis_client_clone = redis_client.clone();
    spawn(async move {
        if let Err(e) = consumer.run(&pool_clone, &redis_client_clone).await {
            error!("Rabbit Worker error: {:?}", e);
        }
    });

    let pool_clone_redis = pool.clone();
    let redis_client_clone_2 = redis_client.clone();
    spawn(async move {
        if let Err(e) = crate::worker::redis_consumer::run_redis_consumer(pool_clone_redis, redis_client_clone_2).await {
            error!("Redis Worker error: {:?}", e);
        }
    });

    tracing::info!("Analytics Service listening on 0.0.0.0:{}", port);

    let jwt_secret = env::var("SECRET").unwrap_or_else(|_| "something".to_string());
    let middleware = TenantAuthMiddleware::with_redis(redis_client.get_ref().clone())
        .with_secret(jwt_secret);

    let _ = HttpServer::new(move || {
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .app_data(pool.clone())
            .app_data(repo.clone())
            .app_data(rabbitconsume.clone())
            .app_data(db_router.clone())
            .app_data(redis_client.clone())
            .wrap(middleware.clone())
            .route(
                "/health",
                web::get().to(handlers::health),
            )
            .route(
                "/analytics",
                web::get().to(handlers::get_analytics),
            )
            .route(
                "/analytics",
                web::post().to(handlers::post_analytics),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await;

    Ok(())
}
