import os

base_dir = r"c:\Users\USER\Documents\Previous\E-commerce\b2b-SaaS-application\rust-backend\user-management"
src_dir = os.path.join(base_dir, "src")
migrations_dir = os.path.join(base_dir, "migrations")

# 1. errors.rs
with open(os.path.join(src_dir, "errors.rs"), "w") as f:
    f.write("""// Domain specific errors for user-management
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found")]
    NotFound,
}
""")

# 2. events.rs
with open(os.path.join(src_dir, "events.rs"), "w") as f:
    f.write("""// Domain events emitted by user-management
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum UserEvent {
    UserCreated { user_id: Uuid, email: String },
    UserDeleted { user_id: Uuid },
}
""")

# 3. redis_sub.rs (fill stub)
with open(os.path.join(src_dir, "redis_sub.rs"), "w") as f:
    f.write("""// Redis Stream subscriber for user-management
pub async fn listen_to_redis_events() {
    // TODO: Implement subscriber logic
}
""")

# 4. sessions migration
with open(os.path.join(migrations_dir, "20260803_create_sessions.sql"), "w") as f:
    f.write("""-- Persistent session/refresh token table
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token VARCHAR(255) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
""")

print("User Management scaffolded successfully.")
