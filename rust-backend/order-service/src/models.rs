use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub id: Uuid,
    pub product_id: Uuid,
    pub restaurant_id: Uuid,
    pub supplier_id: Uuid,
    pub items: serde_json::Value,
    pub qty: Option<i32>
    pub status: UpdateOrderStatus,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub restaurant_id: Uuid,
    pub supplier_id: Uuid,
    pub items: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum UpdateOrderStatus {
    Pending,
    Shipped,
    Delivered,
    Failed
}

// -- products inventory
CREATE TABLE inventory (
    product_id UUID PRIMARY KEY,
    stock integer NOT NULL DEFAULT 0,
    updated_at timestamptz DEFAULT now()
);

// -- reservations
CREATE TABLE reservations (
    reservation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL,
    product_id UUID NOT NULL,
    qty integer NOT NULL,
    expires_at timestamptz,
    created_at timestamptz DEFAULT now(),
    released boolean NOT NULL DEFAULT FALSE
);