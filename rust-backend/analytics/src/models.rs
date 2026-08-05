use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

// =============================
//  ANALYTICS EVENT (RAW EVENT)
// =============================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnalyticsEvent {
    pub tenant_id: Option<Uuid>,
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
    pub tenant_id: Option<Uuid>,
    pub event_type: String,
    pub event_timestamp: Option<DateTime<Utc>>,
    pub data: serde_json::Value,
}

// =============================
// QUERY STRUCT
// (for GET /analytics?...)
// =============================
// Note: AnalyticsRequestQuery is documented in OpenAPI schema; runtime handler uses
// HashMap<String,String> directly for full flexibility over unknown query params.
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
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
#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
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
