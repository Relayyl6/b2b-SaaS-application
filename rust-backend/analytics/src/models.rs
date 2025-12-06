use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

// =============================
//  ANALYTICS EVENT (RAW EVENT)
// =============================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalyticsEvent {
    pub event_type: String,

    pub product_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub order_id: Option<Uuid>,

    // Product-related
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<f64>,
    pub category: Option<String>,
    pub low_stock_threshold: Option<i32>,
    pub unit: Option<String>,
    pub available: Option<bool>,

    // Inventory changes
    pub quantity_change: Option<i32>,

    // Order-related
    pub quantity: Option<i32>,
    pub reservation_id: Option<Uuid>,

    // Timestamps
    pub timestamp: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}


// =============================
// (what is published to RabbitMQ)
// =============================
#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Event {
    pub id: Option<Uuid>,
    pub event_type: String,
    pub event_timestamp: Option<DateTime<Utc>>,
    pub data: serde_json::Value,
}


// =============================
// QUERY STRUCT
// (for GET /analytics?...)
// =============================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalyticsRequestQuery {
    // REQUIRED
    pub metric: String,

    // OPTIONAL
    pub window: Option<String>,
    pub group_by: Option<String>,
    pub aggregate_field: Option<String>,
    pub limit: Option<i64>,
    pub order_by: Option<String>,

    // Non-reserved filters (e.g: &category=food&supplier_id=...)
    pub filters: HashMap<String, String>,
}


// =============================
// REQUEST BODY STRUCT
// (POST body for analytics queries)
// =============================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalyticsRequestBody {
    pub metric: Option<String>,
    pub window: Option<String>,
    pub group_by: Option<String>,
    pub aggregate_field: Option<String>,
    pub limit: Option<i64>,
    pub order_by: Option<String>,

    // Additional dynamic filters
    pub filters: Option<HashMap<String, String>>,
}


// =============================
// MERGED STRUCT (FINAL RESULT)
// =============================
// #[derive(Debug, Clone)]
// pub struct AnalyticsResolvedParams {
//     pub metric: String,
//     pub window: Option<String>,
//     pub group_by: Option<String>,
//     pub aggregate_field: Option<String>,
//     pub limit: Option<i64>,
//     pub order_by: Option<String>,

//     pub filters: HashMap<String, String>,
// }
