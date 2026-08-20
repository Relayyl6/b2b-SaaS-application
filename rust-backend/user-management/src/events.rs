// Domain events emitted by user-management
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum UserEvent {
    UserCreated { user_id: Uuid, email: String },
    UserDeleted { user_id: Uuid },
}
