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
    pub tenant_id: Uuid,
    pub owner_user_id: Uuid,
    pub legal_name: String,
    pub display_name: String,
    pub tax_id: Option<String>,
    pub country: String,
    pub status: SupplierStatus,
    pub stripe_account_id: Option<String>,
    pub platform_fee_percent: f64,
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
    pub platform_fee_percent: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSupplierStatusRequest {
    pub status: SupplierStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSupplierRequest {
    pub legal_name: Option<String>,
    pub display_name: Option<String>,
    pub tax_id: Option<String>,
    pub country: Option<String>,
    pub platform_fee_percent: Option<f64>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierEvent {
    pub tenant_id: Option<Uuid>,
    pub event_type: String,
    pub supplier_id: Uuid,
    pub user_id: Uuid,
    pub owner_user_id: Uuid,
    pub status: SupplierStatus,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supplier_status_serialization() {
        let status = SupplierStatus::Active;
        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(serialized, "\"active\"");

        let deserialized: SupplierStatus = serde_json::from_str("\"rejected\"").unwrap();
        assert_eq!(deserialized, SupplierStatus::Rejected);
    }

    #[test]
    fn test_supplier_event_serialization() {
        let event = SupplierEvent {
            tenant_id: Some(Uuid::nil()),
            event_type: "supplier.created".to_string(),
            supplier_id: Uuid::nil(),
            user_id: Uuid::nil(),
            owner_user_id: Uuid::nil(),
            status: SupplierStatus::Pending,
            timestamp: Utc::now(),
        };

        let json_val = serde_json::to_value(&event).unwrap();
        assert_eq!(json_val["status"], "pending");
        assert_eq!(json_val["event_type"], "supplier.created");
    }
}
