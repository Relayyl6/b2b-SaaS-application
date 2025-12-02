use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::models::Event


// impl Event {
//     pub fn new(
//         event_type: impl Into<String>,
//         data: serde_json::Value,
//         id: Option<Uuid>
//     ) -> Self {
//         Event {
//             event_type: event_type.into(),
//             event_timestamp: Utc::now(),
//             data,
//             id: Some(id),
//         }
//     }
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
fn allowed_group_by(metric: &str) -> &'static [&'static str] {
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
fn parse_window_to_interval(window: &str) -> Option<String> {
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
                "d" | "day" | "days" => return Some(format!("{} days", digits)),
                "h" | "hour" | "hours" => return Some(format!("{} hours", digits)),
                "m" | "min" | "mins" => return Some(format!("{} minutes", digits)),
                _ => return None,
            }
        }
    }
    None
}