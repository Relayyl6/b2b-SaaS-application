use crate::models::Claims;
use crate::models::UserRole;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono;
use jsonwebtoken::{EncodingKey, Header, encode};
use sqlx::PgPool;
use uuid::Uuid;

pub fn hash_password(password: &str) -> String {
    let salt = argon2::password_hash::SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[allow(dead_code)]
pub fn create_jwt(
    user_id: Uuid,
    role: &UserRole,
    secret: &str,
) -> Result<(String, String), jsonwebtoken::errors::Error> {
    create_jwt_with_tenant(
        user_id,
        role,
        Uuid::nil(),
        platform::tenant::PricingTier::Free,
        secret,
    )
}

pub fn create_jwt_with_tenant(
    user_id: Uuid,
    role: &UserRole,
    tenant_id: Uuid,
    tier: platform::tenant::PricingTier,
    secret: &str,
) -> Result<(String, String), jsonwebtoken::errors::Error> {
    let access_exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(15))
        .unwrap()
        .timestamp() as usize;

    let refresh_exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .unwrap()
        .timestamp() as usize;

    let access_claims = Claims {
        sub: user_id,
        role: role.clone(),
        exp: access_exp,
        tenant_id,
        tier,
    };

    let refresh_claims = Claims {
        sub: user_id,
        role: role.clone(),
        exp: refresh_exp,
        tenant_id,
        tier,
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok((access_token, refresh_token))
}

pub async fn user_exists(pool: &PgPool, email: &str, tenant_id: Uuid) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>("SELECT 1 FROM users WHERE email = $1 AND tenant_id = $2")
        .bind(email)
        .bind(tenant_id)
        .fetch_optional(pool)
        .await?;

    Ok(row.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, DecodingKey, Validation};

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "SuperSecretPassword123!";
        let hash = hash_password(password);
        
        assert!(verify_password(&hash, password));
        assert!(!verify_password(&hash, "WrongPassword"));
    }

    #[test]
    fn test_create_jwt() {
        let user_id = Uuid::new_v4();
        let role = UserRole::Admin;
        let secret = "test_secret";
        
        let (access, refresh) = create_jwt(user_id, &role, secret).unwrap();
        
        // verify access token
        let decoded_access = decode::<Claims>(
            &access,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        ).unwrap();
        
        assert_eq!(decoded_access.claims.sub, user_id);
        assert_eq!(decoded_access.claims.role, UserRole::Admin);
        
        // verify refresh token
        let decoded_refresh = decode::<Claims>(
            &refresh,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        ).unwrap();
        
        assert_eq!(decoded_refresh.claims.sub, user_id);
        assert_eq!(decoded_refresh.claims.role, UserRole::Admin);
    }
}
