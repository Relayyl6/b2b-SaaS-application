use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub price: f64,
    pub unit: String,
    pub quantity: i32,
    pub available: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub product_id: Option<Uuid>,
    pub supplier_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub price: f64,
    pub unit: String,
    pub quantity: Option<i32>,
    pub available: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub product_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub price: Option<f64>,
    pub unit: Option<String>,
    pub quantity: Option<i32>,
    pub available: Option<bool>,
    pub quantity_change: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct BulkCreateRequest {
    pub products: Vec<CreateProductRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductEvent {
    pub event_type: String, // "created" | "updated" | "deleted"
    pub product_id: Uuid,
    pub supplier_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub category: Option<String>,
    pub quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub unit: Option<String>,
    pub quantity_change: Option<i32>,
}
