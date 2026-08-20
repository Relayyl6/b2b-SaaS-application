use actix_web::{web, HttpResponse, Responder};
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
