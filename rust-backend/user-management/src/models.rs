use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Users {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub full_name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
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
    pub role: Option<UserRole>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[allow(dead_code)] // used by sign_out handler via JSON deserialisation
#[derive(Serialize, Deserialize, Debug)]
pub struct SignOutRequest {
    pub token: String,
}

#[allow(dead_code)] // response type for auth endpoints
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub user: Users,
    pub token: String,
}

fn default_tenant_id() -> Uuid {
    Uuid::nil()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub role: UserRole,
    pub exp: usize,
    #[serde(default = "default_tenant_id")]
    pub tenant_id: Uuid,
    #[serde(default)]
    pub tier: platform::tenant::PricingTier,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
}

#[allow(dead_code)] // used by delete_user handler via JSON deserialisation
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteUserRequest {
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifyEmailRequest {
    pub token: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_user_role_serialization() {
        assert_eq!(serde_json::to_string(&UserRole::Admin).unwrap(), "\"Admin\"");
        assert_eq!(serde_json::to_string(&UserRole::Supplier).unwrap(), "\"Supplier\"");
        assert_eq!(serde_json::to_string(&UserRole::User).unwrap(), "\"User\"");
    }

    #[test]
    fn test_user_role_deserialization() {
        assert_eq!(serde_json::from_str::<UserRole>("\"Admin\"").unwrap(), UserRole::Admin);
        assert_eq!(serde_json::from_str::<UserRole>("\"Supplier\"").unwrap(), UserRole::Supplier);
        assert_eq!(serde_json::from_str::<UserRole>("\"User\"").unwrap(), UserRole::User);
    }

    #[test]
    fn test_signup_request_deserialization() {
        let json_data = json!({
            "email": "test@example.com",
            "password": "Password123",
            "full_name": "Test User",
            "role": "Admin"
        });
        let req: SignUpRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.role, Some(UserRole::Admin));
    }

    #[test]
    fn test_signup_request_deserialization_no_role() {
        let json_data = json!({
            "email": "test@example.com",
            "password": "Password123",
            "full_name": "Test User"
        });
        let req: SignUpRequest = serde_json::from_value(json_data).unwrap();
        assert_eq!(req.email, "test@example.com");
        assert_eq!(req.role, None);
    }
    
    #[test]
    fn test_claims_serialization() {
        let id = Uuid::new_v4();
        let claims = Claims {
            sub: id,
            role: UserRole::User,
            exp: 1234567890,
            tenant_id: Uuid::nil(),
            tier: platform::tenant::PricingTier::Free,
        };
        let serialized = serde_json::to_string(&claims).unwrap();
        assert!(serialized.contains(&id.to_string()));
        assert!(serialized.contains("User"));
        assert!(serialized.contains("1234567890"));
    }
}
