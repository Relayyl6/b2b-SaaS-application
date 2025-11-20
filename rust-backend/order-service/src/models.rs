use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub id: Uuid,
    pub product_id: Uuid,
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub items: serde_json::Value,
    pub qty: Option<i32>
    pub status: OrderStatus,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub qty: i32,
    pub items: serde_json::Value,
}

pub struct UpdateOrderStatus {
    pub product_id: Uuid,
    pub new_status: OrderStatus,
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

// -- products inventor