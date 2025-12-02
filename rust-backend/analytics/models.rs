
/// Request query params (all optional)
#[derive(Deserialize)]
pub struct AnalyticsQuery {
    pub metric: String,                    // required -> short name e.g. "signups"
    pub window: Option<String>,            // e.g. "7d", "30d", "90d"
    pub group_by: Option<String>,          // comma separated, e.g. "signup_source,country"
    pub aggregate_field: Option<String>,   // e.g. "signups" (defaults per metric)
    pub limit: Option<i64>,
    pub order_by: Option<String>,          // e.g. "value_desc" or "day_desc"
    // Generic filters (passed as query params; handled generically below)
    // e.g. ?country=NG&signup_source=web
}

/// Also accept JSON body as fallback (same shape)
#[derive(Deserialize)]
pub struct AnalyticsBody {
    pub metric: Option<String>,
    pub window: Option<String>,
    pub group_by: Option<String>,
    pub aggregate_field: Option<String>,
    pub limit: Option<i64>,
    pub order_by: Option<String>,
    // generic filters as an object
    pub filters: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub event_type: String,
    pub event_timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
    pub id: Option<Uuid>,
}