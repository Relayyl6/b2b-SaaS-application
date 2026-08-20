use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;

use platform::{metrics, observability};

mod auth;
mod models;
mod handlers;
mod routes;
mod db;
mod errors;
mod events;

use models::*;

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        handlers::create_tenant,
        handlers::generate_api_key_handler,
        handlers::health,
        handlers::metrics_api_doc
    ),
    components(
        schemas(HealthResponse, CreateTenantRequest, TenantResponse, GenerateKeyRequest, GenerateKeyResponse)
    )
)]
pub struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    observability::init_observability("tenant-management");
    metrics::init_metrics("tenant-management");

    let db_url = env::var("CONTROL_PLANE_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://control_user:commerce_secret@localhost:5433/commerce_control".to_string());
    
    let pool = PgPool::connect(&db_url).await.expect("Failed to connect to control plane DB");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let port = env::var("TENANT_MANAGEMENT_PORT")
        .unwrap_or_else(|_| "3000".to_string());

    tracing::info!("Tenant Management Service listening on 0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(handlers::health))
            .route("/metrics", web::get().to(metrics::metrics_handler))
            .configure(routes::configure)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
