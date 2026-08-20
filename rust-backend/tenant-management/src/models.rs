use serde::{Deserialize, Serialize};
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
