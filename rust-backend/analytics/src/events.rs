use crate::models::{AnalyticsEvent, Event};
use thiserror::Error;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("invalid event data: missing key {0}")]
    MissingKey(String),

    #[error("invalid uuid format")]
    InvalidUuid,

    #[error("generic conversion error: {0}")]
    ConversionError(String),
}

impl AnalyticsEvent {
    /// Get the ID associated with the event_type (order_id, product_id, etc.)
    pub fn extract_primary_id(
        &self
    ) -> Uuid {
        if self.event_type.starts_with("order.") {
            return self.order_id.unwrap_or_else(Uuid::new_v4);
        }
        if self.event_type.starts_with("product.") {
            return self.product_id.unwrap_or_else(Uuid::new_v4);
        }
        if self.event_type.starts_with("user.") {
            return self.user_id.unwrap_or_else(Uuid::new_v4);
        }
        if self.event_type.starts_with("inventory.") {
            return self.supplier_id.unwrap_or_else(Uuid::new_v4);
        }
        // default for unknown events
        Uuid::new_v4()
    }
}


impl Event {
    pub fn new(
        event: AnalyticsEvent
    ) -> Result<Event, EventError> {
        let id = event.extract_primary_id();

        let data = serde_json::to_value(&event)
            .map_err(|e| EventError::ConversionError(e.to_string()))?;

        Ok(
            Event {
                event_type: event.event_type,
                event_timestamp: Some(event.timestamp.unwrap_or(Utc::now())),
                data,
                id: Some(id),
            }
        )
    }
}



/// Allowed metrics -> underlying table mapping
pub async fn metric_table_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("signups", "analytics.user_signups_daily");
    m.insert("orders", "analytics.orders_daily");
    m.insert("revenue", "analytics.revenue_daily");
    m.insert("product_views", "analytics.product_views_daily");
    m.insert("product_metrics", "analytics.product_metrics_daily");
    m.insert("inventory", "analytics.inventory_daily");
    m.insert("delivery", "analytics.delivery_performance_daily");
    m.insert("payments", "analytics.payments_daily");
    m.insert("notifications", "analytics.notifications_daily");
    m.insert("top_products_7d", "analytics.top_products_7d");
    m
}


/// Whitelist of allowed group_by columns per metric (prevents injection and invalid columns)
pub fn allowed_group_by(metric: &str) -> &'static [&'static str] {
    match metric {
        "signups" => &["signup_source", "signup_platform", "country", "day"],
        "orders" => &["day", "order_id_sample"],
        "revenue" => &["day"],
        "product_views" | "product_metrics" => &["product_id", "day"],
        "inventory" => &["product_id", "day"],
        "delivery" => &["carrier", "day"],
        "payments" => &["payment_method", "day"],
        "notifications" => &["channel", "day"],
        _ => &["day"],
    }
}

/// Convert short window like "30d" -> SQL interval string "30 days"
pub fn parse_window_to_interval(window: &str) -> Option<String> {
    // very small parser: digits + suffix (d|h|m)
    if window.is_empty() {
        return None;
    }
    let mut chars = window.chars();
    let mut digits = String::new();
    while let Some(c) = chars.next() {
        if c.is_digit(10) { digits.push(c); } else {
            let rest: String = std::iter::once(c).chain(chars).collect();
            match rest.as_str() {
                "mo" | "month" | "months" => return Some(format!("{} months", digits)),
                "d" | "day" | "days" => return Some(format!("{} days", digits)),
                "h" | "hour" | "hours" => return Some(format!("{} hours", digits)),
                "m" | "min" | "mins" => return Some(format!("{} minutes", digits)),
                _ => return None,
            }
        }
    }
    None
}