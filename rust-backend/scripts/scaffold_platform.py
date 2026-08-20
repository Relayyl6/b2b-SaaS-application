import os

base_dir = r"c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend\platform"
src_dir = os.path.join(base_dir, "src")
middleware_dir = os.path.join(src_dir, "middleware")
tests_dir = os.path.join(base_dir, "tests")

# Ensure directories exist
os.makedirs(middleware_dir, exist_ok=True)
os.makedirs(tests_dir, exist_ok=True)

# 1. errors.rs
with open(os.path.join(src_dir, "errors.rs"), "w") as f:
    f.write("""use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error")]
    Internal,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_type) = match self {
            AppError::Database(_) | AppError::Internal => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            AppError::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "not_found"),
            AppError::Unauthorized(_) => (actix_web::http::StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::Forbidden(_) => (actix_web::http::StatusCode::FORBIDDEN, "forbidden"),
            AppError::BadRequest(_) => (actix_web::http::StatusCode::BAD_REQUEST, "bad_request"),
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        })
    }
}
""")

# 2. config.rs
with open(os.path.join(src_dir, "config.rs"), "w") as f:
    f.write("""use std::env;

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub database_url: String,
    pub redis_url: String,
    pub service_port: u16,
    pub amqp_addr: Option<String>,
}

impl ServiceConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/db".to_string()),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            service_port: env::var("SERVICE_PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap_or(8080),
            amqp_addr: env::var("AMQP_ADDR").ok(),
        }
    }
}
""")

# 3. events.rs
with open(os.path.join(src_dir, "events.rs"), "w") as f:
    f.write("""use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub event_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum DomainEvent {
    OrderCreated { order_id: Uuid, amount: f64 },
    PaymentSucceeded { payment_id: Uuid, order_id: Uuid },
    ProductCreated { product_id: Uuid },
    // Expand as needed
}
""")

# 4. pagination.rs
with open(os.path.join(src_dir, "pagination.rs"), "w") as f:
    f.write("""use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}
""")

# 5. health.rs
with open(os.path.join(src_dir, "health.rs"), "w") as f:
    f.write("""use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
    pub version: &'static str,
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthStatus {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check));
}
""")

# Middlewares
with open(os.path.join(middleware_dir, "request_id.rs"), "w") as f:
    f.write("""// Stub for Request ID middleware
pub struct RequestIdMiddleware;
// TODO: Implement actix_web Transform
""")

with open(os.path.join(middleware_dir, "rate_limiter.rs"), "w") as f:
    f.write("""// Stub for Redis Rate Limiter middleware
pub struct RateLimiterMiddleware;
// TODO: Implement actix_web Transform
""")

with open(os.path.join(middleware_dir, "idempotency.rs"), "w") as f:
    f.write("""// Stub for Idempotency middleware
pub struct IdempotencyMiddleware;
// TODO: Implement actix_web Transform
""")

# Update middleware/mod.rs
with open(os.path.join(middleware_dir, "mod.rs"), "a") as f:
    f.write("""
pub mod request_id;
pub mod rate_limiter;
pub mod idempotency;
""")

# Update lib.rs
with open(os.path.join(src_dir, "lib.rs"), "a") as f:
    f.write("""
pub mod errors;
pub mod config;
pub mod events;
pub mod pagination;
pub mod health;
""")

# Test stubs
with open(os.path.join(tests_dir, "middleware_tests.rs"), "w") as f:
    f.write("""#[cfg(test)]
mod tests {
    #[test]
    fn test_tenant_auth_middleware_valid_jwt() {
        // TODO: test valid JWT
    }
}
""")

with open(os.path.join(tests_dir, "rls_tests.rs"), "w") as f:
    f.write("""#[cfg(test)]
mod tests {
    #[test]
    fn test_rls_prevents_cross_tenant_access() {
        // TODO: test apply_rls logic
    }
}
""")

print("Platform primitives scaffolded successfully.")
