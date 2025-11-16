use argon2::{self, Config};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
// use serde::{Deserialize, Serialize};
use chrono;
use crate::models::{UserRole};
use sqlx::PgPool;
use crate::models::Claims;
use uuid::Uuid;

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;

pub fn hash_password(password: &str) -> String {
    let salt = argon2::password_hash::SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

// pub fn generate_random_salt() -> Vec<u8> {
//     let mut salt = vec![0u8; 16]; // 16 bytes salt (128 bits)
//     thread_rng().fill_bytes(&mut salt);
//     salt
// }

pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn create_jwt(user_id: Uuid, role: &UserRole, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
    .checked_add_signed(chrono::Duration::hours(24))
    .unwrap()
    .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        role: role.clone(),
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, String> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
}

pub async fn user_exists(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
            "SELECT 1 FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;
    
    Ok(row.is_some())
}

