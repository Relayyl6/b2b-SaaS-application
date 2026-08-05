use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "payment_status", rename_all = "snake_case")]
pub enum PaymentStatus {
    RequiresPaymentMethod,
    Processing,
    Succeeded,
    Failed,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct PaymentIntent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub idempotency_key: String,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub amount: i64,
    pub currency: String,
    pub provider: String,
    pub provider_reference: Option<String>,
    pub status: PaymentStatus,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreatePaymentIntentRequest {
    pub idempotency_key: String,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub amount: i64,
    pub currency: Option<String>,
    pub provider: Option<String>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct PaymentWebhook {
    pub provider_reference: Option<String>,
    pub idempotency_key: Option<String>,
    pub status: PaymentStatus,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentEvent {
    pub tenant_id: Uuid,
    pub event_type: String,
    pub payment_id: Uuid,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub amount: i64,
    pub currency: String,
    pub provider: String,
    pub provider_reference: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_status_serialization() {
        assert_eq!(
            serde_json::to_string(&PaymentStatus::RequiresPaymentMethod).unwrap(),
            "\"requires_payment_method\""
        );
        assert_eq!(
            serde_json::to_string(&PaymentStatus::Processing).unwrap(),
            "\"processing\""
        );
        assert_eq!(
            serde_json::to_string(&PaymentStatus::Succeeded).unwrap(),
            "\"succeeded\""
        );
    }
}
