use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Inventory {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub description: serde_json::Value,
    pub category: String,
    pub price: f64,
    pub quantity: i32,
    pub low_stock_threshold: i32,
    pub unit: String,
    pub available: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UpdateStockRequest {
    pub product_id: Uuid,
    pub name: Option<String>,
    pub description: Option<serde_json::Value>,
    pub category: Option<String>,
    pub price: Option<f64>,
    pub unit: Option<String>,
    pub quantity: Option<i32>,
    pub quantity_change: Option<i32>,
    pub available: Option<bool>,
    pub low_stock_threshold: Option<i32>,
    pub reserved: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockUpdateEvent {
    pub tenant_id: Option<Uuid>,
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub new_quantity: i32,
    pub change: Option<i32>,
    pub low_stock: bool,
    pub name: Option<String>,
    pub description: Option<serde_json::Value>,
    pub category: Option<String>,
    pub price: Option<f64>,
    pub unit: Option<String>,
    pub available: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInventoryRequest {
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub category: String,
    pub description: serde_json::Value,
    pub price: f64,
    pub quantity: i32,
    pub low_stock_threshold: i32,
    pub unit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ProductEvent {
    pub tenant_id: Option<Uuid>,
    pub event_type: String,
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub name: Option<String>,
    pub description: Option<serde_json::Value>,
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
    pub order_timestamp: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>, // pub status: OrderStatus,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ExpiredReservationRow {
    pub reservation_id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub qty: i32,
    pub user_id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ReservationRow {
    pub reservation_id: Uuid,
    pub qty: i32,
    pub released: bool,
    pub user_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_stock_request_default() {
        let req = UpdateStockRequest::default();
        assert_eq!(req.quantity, None);
        assert_eq!(req.quantity_change, None);
    }

    #[test]
    fn test_product_event_default() {
        let event = ProductEvent::default();
        assert_eq!(event.event_type, "");
        assert_eq!(event.name, None);
        assert_eq!(event.quantity_change, None);
    }
}
