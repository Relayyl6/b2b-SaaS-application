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
    pub description: String,
    pub category: String,
    pub price: f64,
    pub quantity: i32,
    pub low_stock_threshold: i32,
    pub unit: String,
    pub available: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStockRequest {
    pub product_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub price: Option<f64>,
    pub unit: Option<String>,
    pub quantity: Option<i32>,
    pub quantity_change: Option<i32>,
    pub available: Option<bool>,
    pub low_stock_threshold: Option<i32>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StockUpdateEvent {
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub new_quantity: i32,
    pub change: Option<i32>,
    pub low_stock: bool,
    pub name: Option<String>,
    pub description: Option<String>,
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
    pub description: String,
    pub price: f64,
    pub quantity: i32,
    pub low_stock_threshold: i32,
    pub unit: String
}