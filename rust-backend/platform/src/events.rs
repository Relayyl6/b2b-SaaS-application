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

impl<T: Serialize> EventEnvelope<T> {
    pub fn new(tenant_id: Option<Uuid>, correlation_id: Option<Uuid>, payload: T) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            tenant_id,
            correlation_id,
            occurred_at: Utc::now(),
            payload,
        }
    }

    /// Strict schema validation before dispatch
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.event_id.is_nil() {
            return Err("Event ID cannot be nil");
        }
        
        // Ensure payload can cleanly serialize without panic
        let _val = serde_json::to_value(&self.payload).map_err(|_| "Payload serialization failed")?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum DomainEvent {
    OrderCreated { order_id: Uuid, amount: f64 },
    PaymentSucceeded { payment_id: Uuid, order_id: Uuid },
    ProductCreated { product_id: Uuid },
}
