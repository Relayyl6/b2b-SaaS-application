use serde::{Serialize, Deserialize};
use uuid::Uuid;
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Inventory {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub quantity: i32,
    pub low_stock_threshold: i32,
    pub unit: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStockRequest {
    pub product_id: Uuid,
    pub quantity_change: i32,
}

#[derive(Debug, Serialize)]
pub struct StockUpdateEvent {
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub new_quantity: i32,
    pub change: i32,
    pub low_stock: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateInventoryRequest {
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub name: String,
    pub quantity: i32,
    pub low_stock_threshold: i32,
    pub unit: String,
}