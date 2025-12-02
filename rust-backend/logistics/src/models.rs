use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_type: String,
    pub id: Option<Uuid>,
    pub data: json_serde::Value,
    pub event_timestamp: DataTime<Utc>,
}