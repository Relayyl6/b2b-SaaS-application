use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Users {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
#[serde(rename_all=lowercase)]
pub enum UserRole {
    Admin,
    Supplier,
    User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub role: Option<UserRole>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub password: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteUserRequest {
    pub user_id: String,
}