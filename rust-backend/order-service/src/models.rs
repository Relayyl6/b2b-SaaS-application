use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub id: Uuid,
    pub restaurant_id: Uuid,
    pub supplier_id: Uuid,
    pub items: serde_json::Value,
    pub status: String,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub restaurant_id: Uuid,
    pub supplier_id: Uuid,
    pub items: serde_json::Value,
}
