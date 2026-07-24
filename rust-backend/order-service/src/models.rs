use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub product_id: Uuid,
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub items: serde_json::Value,
    pub qty: Option<i32>,
    pub status: OrderStatus,
    pub updated_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub order_timestamp: DateTime<Utc>,
    pub version: i32,
}

// items is basically the name of whatever you ordered
#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub qty: i32,
    pub status: Option<OrderStatus>,
    pub items: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateOrderStatus {
    #[allow(dead_code)] // deserialized from request body; reserved for validation/future use
    pub id: Uuid,
    pub product_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub new_status: Option<OrderStatus>,
    pub expires_at: Option<DateTime<Utc>>,
    pub order_timestamp: Option<DateTime<Utc>>,
    pub expected_version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Processing,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
    Failed,
    Refunded,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OrderEvent {
    pub event_type: String,
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub category: Option<String>,
    pub low_stock_threshold: Option<i32>,
    pub unit: Option<String>,
    pub quantity_change: Option<i32>,
    pub available: Option<bool>,
    // Order-related
    pub order_id: Option<Uuid>,
    pub quantity: Option<i32>,
    pub reservation_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub order_timestamp: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    // pub status: OrderStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_status_serialization() {
        assert_eq!(serde_json::to_string(&OrderStatus::Pending).unwrap(), "\"pending\"");
        assert_eq!(serde_json::to_string(&OrderStatus::Confirmed).unwrap(), "\"confirmed\"");
        
        let status: OrderStatus = serde_json::from_str("\"shipped\"").unwrap();
        assert_eq!(status, OrderStatus::Shipped);
    }
}

