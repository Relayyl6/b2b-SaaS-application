use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "supplier_status", rename_all = "lowercase")]
pub enum SupplierStatus {
    Pending,
    Active,
    Suspended,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Supplier {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub legal_name: String,
    pub display_name: String,
    pub tax_id: Option<String>,
    pub country: String,
    pub status: SupplierStatus,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSupplierRequest {
    pub owner_user_id: Uuid,
    pub legal_name: String,
    pub display_name: String,
    pub tax_id: Option<String>,
    pub country: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSupplierStatusRequest {
    pub status: SupplierStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct SupplierEvent {
    pub event_type: String,
    pub supplier_id: Uuid,
    pub user_id: Uuid,
    pub owner_user_id: Uuid,
    pub status: SupplierStatus,
    pub timestamp: DateTime<Utc>,
}
