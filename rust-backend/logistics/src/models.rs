use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "shipment_status", rename_all = "lowercase")]
pub enum ShipmentStatus {
    Pending,
    Intransit,
    Delivered,
    Cancelled,
}

impl ShipmentStatus {
    /// Returns whether this status may transition to `next` according to the domain rules.
    ///
    /// # Returns
    ///
    /// `true` if the current status is the same as `next` or if moving from the current status
    /// to `next` is allowed (Pending -> Intransit, Pending -> Cancelled, Intransit -> Delivered,
    /// Intransit -> Cancelled), `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::models::ShipmentStatus;
    ///
    /// assert!(ShipmentStatus::Pending.can_transition_to(&ShipmentStatus::Intransit));
    /// assert!(ShipmentStatus::Intransit.can_transition_to(&ShipmentStatus::Delivered));
    /// assert!(!ShipmentStatus::Delivered.can_transition_to(&ShipmentStatus::Pending));
    /// ```
    pub fn can_transition_to(&self, next: &ShipmentStatus) -> bool {
        match (self, next) {
            (ShipmentStatus::Pending, ShipmentStatus::Intransit)
            | (ShipmentStatus::Pending, ShipmentStatus::Cancelled)
            | (ShipmentStatus::Intransit, ShipmentStatus::Delivered)
            | (ShipmentStatus::Intransit, ShipmentStatus::Cancelled) => true,
            (a, b) if a == b => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Shipment {
    pub id: Uuid,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub tracking_number: String,
    pub status: ShipmentStatus,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dispatched_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShipmentRequest {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShipmentStatusRequest {
    pub status: ShipmentStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListShipmentQuery {
    pub status: Option<ShipmentStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticsEvent {
    pub event_type: String,
    pub shipment_id: Uuid,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
    pub status: ShipmentStatus,
    pub tracking_number: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IncomingOrderEvent {
    pub event_type: String,
    pub order_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub supplier_id: Uuid,
    pub product_id: Uuid,
}
