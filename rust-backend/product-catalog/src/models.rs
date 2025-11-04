use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub price: f64, // small convenience mapping from NUMERIC to f64
    pub unit: String,
    pub available: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub supplier_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub price: f64,
    pub unit: String,
    pub available: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub price: Option<f64>,
    pub unit: Option<String>,
    pub available: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct BulkCreateRequest {
    pub products: Vec<CreateProductRequest>,
}

#[derive(Debug, Serialize)]
pub struct ProductEvent {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub event_type: String, // "created" | "updated" | "deleted"
}
