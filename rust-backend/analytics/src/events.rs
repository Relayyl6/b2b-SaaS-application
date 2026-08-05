use crate::models::{AnalyticsEvent, Event};
use chrono::Utc;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
#[allow(dead_code)]
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
    pub fn extract_primary_id(&self) -> Uuid {
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
        if self.event_type.starts_with("logistics.") {
            // No shipment_id available in AnalyticsEvent, return a new UUID
            return Uuid::new_v4();
        }
        // default for unknown events
        Uuid::new_v4()
    }
}

impl Event {
    pub fn new(event: AnalyticsEvent) -> Result<Event, EventError> {
        let id = event.extract_primary_id();
        let tenant_id = event.tenant_id;

        let data =
            serde_json::to_value(&event).map_err(|e| EventError::ConversionError(e.to_string()))?;

        Ok(Event {
            id: Some(id),
            tenant_id,
            event_type: event.event_type,
            event_timestamp: Some(event.timestamp.unwrap_or(Utc::now())),
            data,
        })
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
        if c.is_digit(10) {
            digits.push(c);
        } else {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_primary_id() {
        let mut ev = AnalyticsEvent {
            event_type: "order.created".to_string(),
            tenant_id: None,
            order_id: Some(Uuid::new_v4()),
            product_id: None,
            supplier_id: None,
            user_id: None,
            name: None,
            description: None,
            price: None,
            category: None,
            low_stock_threshold: None,
            unit: None,
            available: None,
            quantity_change: None,
            quantity: None,
            reservation_id: None,
            timestamp: None,
            expires_at: None,
        };
        assert_eq!(ev.extract_primary_id(), ev.order_id.unwrap());

        ev.event_type = "product.updated".to_string();
        ev.product_id = Some(Uuid::new_v4());
        assert_eq!(ev.extract_primary_id(), ev.product_id.unwrap());

        ev.event_type = "user.created".to_string();
        ev.user_id = Some(Uuid::new_v4());
        assert_eq!(ev.extract_primary_id(), ev.user_id.unwrap());
    }

    #[tokio::test]
    async fn test_metric_table_map() {
        let map = metric_table_map().await;
        assert_eq!(map.get("signups"), Some(&"analytics.user_signups_daily"));
        assert_eq!(map.get("revenue"), Some(&"analytics.revenue_daily"));
        assert!(map.get("unknown_metric").is_none());
    }

    #[test]
    fn test_allowed_group_by() {
        assert_eq!(allowed_group_by("signups"), &["signup_source", "signup_platform", "country", "day"]);
        assert_eq!(allowed_group_by("unknown_metric"), &["day"]);
    }

    #[test]
    fn test_parse_window_to_interval() {
        assert_eq!(parse_window_to_interval("30d"), Some("30 days".to_string()));
        assert_eq!(parse_window_to_interval("12h"), Some("12 hours".to_string()));
        assert_eq!(parse_window_to_interval("6mo"), Some("6 months".to_string()));
        assert_eq!(parse_window_to_interval("invalid"), None);
    }
}
