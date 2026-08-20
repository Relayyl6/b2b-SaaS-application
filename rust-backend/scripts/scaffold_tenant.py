import os

base_dir = r"c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend\tenant-management"
src_dir = os.path.join(base_dir, "src")
migrations_dir = os.path.join(base_dir, "migrations")

os.makedirs(migrations_dir, exist_ok=True)

# 1. models.rs
with open(os.path.join(src_dir, "models.rs"), "w") as f:
    f.write("""use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    pub name: String,
    pub email: String,
    pub tier: Option<String>,
}

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct TenantResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub tier: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GenerateKeyRequest {
    pub tenant_id: Uuid,
    pub name: String,
    pub key_type: String, // "sk" or "pk"
    pub environment: String, // "live" or "test"
    pub scopes: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct GenerateKeyResponse {
    pub plaintext_key: String,
    pub prefix: String,
    pub key_type: String,
    pub environment: String,
}
""")

# 2. handlers.rs
with open(os.path.join(src_dir, "handlers.rs"), "w") as f:
    f.write("""use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use crate::models::*;
use crate::auth;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check OK", body = HealthResponse)
    )
)]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics")
    )
)]
#[allow(dead_code)]
pub async fn metrics_api_doc() {}

#[utoipa::path(
    post,
    path = "/v1/tenants",
    request_body = CreateTenantRequest,
    responses(
        (status = 201, description = "Tenant created", body = TenantResponse),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("BearerAuth" = []),
        ("ApiKeyAuth" = [])
    )
)]
pub async fn create_tenant(
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

#[utoipa::path(
    post,
    path = "/v1/tenants/keys",
    request_body = GenerateKeyRequest,
    responses(
        (status = 201, description = "Key generated", body = GenerateKeyResponse),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("BearerAuth" = []),
        ("ApiKeyAuth" = [])
    )
)]
pub async fn generate_api_key_handler(
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
""")

# 3. routes.rs
with open(os.path.join(src_dir, "routes.rs"), "w") as f:
    f.write("""use actix_web::web;
use crate::handlers::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/v1/tenants", web::post().to(create_tenant))
       .route("/v1/tenants/keys", web::post().to(generate_api_key_handler));
}
""")

# 4. db.rs, errors.rs, events.rs (Stubs for now)
with open(os.path.join(src_dir, "db.rs"), "w") as f:
    f.write("// Database repository patterns for tenant-management\n")
with open(os.path.join(src_dir, "errors.rs"), "w") as f:
    f.write("// Domain specific errors for tenant-management\n")
with open(os.path.join(src_dir, "events.rs"), "w") as f:
    f.write("// Domain events emitted by tenant-management\n")

# 5. main.rs
with open(os.path.join(src_dir, "main.rs"), "w") as f:
    f.write("""use utoipa::OpenApi;
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
""")

# Migrations
with open(os.path.join(migrations_dir, "20260802_add_rls_policies.sql"), "w") as f:
    f.write("""-- Add RLS policies for tenants
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;
CREATE POLICY api_keys_tenant_isolation_policy ON api_keys
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
""")

with open(os.path.join(migrations_dir, "20260803_add_api_keys_indexes.sql"), "w") as f:
    f.write("""-- Add composite indexes for lookup performance
CREATE INDEX IF NOT EXISTS idx_api_keys_tenant_env ON api_keys(tenant_id, environment) WHERE is_active = TRUE;
""")

with open(os.path.join(migrations_dir, "20260804_add_tenant_webhooks.sql"), "w") as f:
    f.write("""-- Webhook configuration table
CREATE TABLE tenant_webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    secret VARCHAR(255) NOT NULL,
    events TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
ALTER TABLE tenant_webhooks ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_webhooks_isolation_policy ON tenant_webhooks
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);
""")

print("Tenant Management scaffolded successfully.")
