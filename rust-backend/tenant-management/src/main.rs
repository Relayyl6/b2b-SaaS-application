use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;

use platform::{metrics, observability};
use uuid::Uuid;

mod auth;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[derive(Deserialize)]
struct CreateTenantRequest {
    name: String,
    email: String,
    tier: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
struct TenantResponse {
    id: Uuid,
    name: String,
    email: String,
    tier: String,
}

async fn create_tenant(
    pool: web::Data<PgPool>,
    req: web::Json<CreateTenantRequest>,
) -> impl Responder {
    let tier_str = req.tier.as_deref().unwrap_or("free");

    let row = sqlx::query_as::<_, TenantResponse>(
        r#"
        INSERT INTO tenants (name, email, tier)
        VALUES ($1, $2, $3)
        RETURNING id, name, email, tier
        "#
    )
    .bind(&req.name)
    .bind(&req.email)
    .bind(tier_str)
    .fetch_one(pool.get_ref())
    .await;

    match row {
        Ok(tenant) => HttpResponse::Created().json(tenant),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(Deserialize)]
struct GenerateKeyRequest {
    tenant_id: Uuid,
    name: String,
    key_type: String, // "sk" or "pk"
    environment: String, // "live" or "test"
    scopes: Vec<String>,
}

#[derive(Serialize)]
struct GenerateKeyResponse {
    plaintext_key: String, // ONLY TIME THIS IS EVER SHOWN
    prefix: String,
    key_type: String,
    environment: String,
}

async fn generate_api_key_handler(
    pool: web::Data<PgPool>,
    req: web::Json<GenerateKeyRequest>,
) -> impl Responder {
    let api_key = auth::generate_api_key(&req.key_type, &req.environment);

    let row = sqlx::query(
        r#"
        INSERT INTO api_keys (tenant_id, name, key_prefix, key_hash, key_type, environment, scopes)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#
    )
    .bind(req.tenant_id)
    .bind(&req.name)
    .bind(api_key.prefix.clone())
    .bind(api_key.hash.clone())
    .bind(&req.key_type)
    .bind(&req.environment)
    .bind(&req.scopes)
    .execute(pool.get_ref())
    .await;

    match row {
        Ok(_) => HttpResponse::Created().json(GenerateKeyResponse {
            plaintext_key: api_key.plaintext,
            prefix: api_key.prefix.clone(),
            key_type: req.key_type.clone(),
            environment: req.environment.clone(),
        }),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    observability::init_observability("tenant-management");
    metrics::init_metrics("tenant-management");

    let db_url = env::var("CONTROL_PLANE_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://control_user:commerce_secret@localhost:5433/commerce_control".to_string());
    
    let pool = PgPool::connect(&db_url).await.expect("Failed to connect to control plane DB");

    // Run migrations automatically
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let port = env::var("TENANT_MANAGEMENT_PORT")
        .unwrap_or_else(|_| "3000".to_string());

    tracing::info!("Tenant Management Service listening on 0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health))
            .route("/metrics", web::get().to(metrics::metrics_handler))
            .route("/v1/tenants", web::post().to(create_tenant))
            .route("/v1/tenants/keys", web::post().to(generate_api_key_handler))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
