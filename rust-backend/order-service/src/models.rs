use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub id: Uuid,
    pub product_id: Uuid,
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub items: serde_json::Value,
    pub qty: Option<i32>,
    pub status: OrderStatus,
    pub expires_at: Option<i64>,
    pub timestamp: Option<i64>,
}

// items is basically the name of whatever you ordered
#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub qty: i32,
    pub items: serde_json::Value,
}

pub struct UpdateOrderStatus {
    pub id: Uuid,
    pub product_id: Uuid,
    pub user_id: Option<Uuid>,
    pub new_status: OrderStatus,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
    Failed
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub timestamp: Option<i64>,
    pub expires_at: Option<i64>,
    pub user_id: Option<Uuid>,
    // pub status: OrderStatus,
}

// -- products inventor