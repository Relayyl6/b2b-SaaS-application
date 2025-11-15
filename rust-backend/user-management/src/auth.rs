use argon2::{self, Config};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
// use serde::{Deserialize, Serialize};
use chrono;
use crate::models::Users;
use sqlx::PgPool;
use rand::{thread_rng, RngCore};
use crate::models::UserRole

pub fn hash_password(password: &str) -> String {
    let salt = generate_random_salt();
    let config = Config::default();
    argon2::hash_encoded(password.as_bytes(), &salt, &config).expect("Failed to hash password")
}


pub fn generate_random_salt() -> Vec<u8> {
    let mut salt = vec![0u8; 16]; // 16 bytes salt (128 bits)
    thread_rng().fill_bytes(&mut salt);
    salt
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    return argon2::verify_encoded(hash, password.as_bytes()).unwrap_or(false);
}

struct Claims {
    sub: i32,
    role: &static str,
    exp: usize
}

pub fn create_jwt(user_id: i32, role: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = chrono::Utc::now()
    .checked_add_signed(chrono::duration::hours(24))
    .unwrap()
    .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        role: UserRole,
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()));
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

pub async fn auth_middle_ware() -> Result<> {

}




