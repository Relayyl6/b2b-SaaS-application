use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub event_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
    pub payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum DomainEvent {
    OrderCreated { order_id: Uuid, amount: f64 },
    PaymentSucceeded { payment_id: Uuid, order_id: Uuid },
    ProductCreated { product_id: Uuid },
    // Expand as needed
}
