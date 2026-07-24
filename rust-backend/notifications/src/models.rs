use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "notification_channel", rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
    Sms,
    Push,
    InApp,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "notification_status", rename_all = "lowercase")]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "notification_priority", rename_all = "lowercase")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub event_type: String,
    pub channel: NotificationChannel,
    pub priority: NotificationPriority,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
    pub payload: Value,
    pub status: NotificationStatus,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub channel: NotificationChannel,
    pub priority: Option<NotificationPriority>,
    pub recipient: Option<String>,
    pub subject: Option<String>,
    pub body: String,
    pub payload: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub user_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub status: Option<NotificationStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct DomainEvent {
    pub event_type: Option<String>,
    pub user_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub quantity: Option<i32>,
    pub status: Option<String>,
    pub tracking_number: Option<String>,
    pub recipient: Option<String>,
    #[serde(flatten)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "notification_device_platform", rename_all = "lowercase")]
pub enum DevicePlatform {
    Ios,
    Android,
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationDevice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: DevicePlatform,
    pub push_token: String,
    pub provider: String,
    pub device_id: Option<String>,
    pub app_version: Option<String>,
    pub enabled: bool,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterDeviceRequest {
    pub user_id: Uuid,
    pub platform: DevicePlatform,
    pub push_token: String,
    pub provider: Option<String>,
    pub device_id: Option<String>,
    pub app_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreference {
    pub user_id: Uuid,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub in_app_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub email_enabled: Option<bool>,
    pub sms_enabled: Option<bool>,
    pub push_enabled: Option<bool>,
    pub in_app_enabled: Option<bool>,
}
