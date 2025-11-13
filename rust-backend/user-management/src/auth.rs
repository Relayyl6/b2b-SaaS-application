use argon2::{self, Config};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use chrono;

pub fn hash_password(password: &str) -> String {
    let salt = generate_random_salt();
    let config = Config::default();
    argon2::hash_encoded(password.as_bytes(), salt, &config).unwrap();
}


pub fn generate_random_salt() -> String {

}

pub fn verify_password(hash: &str, password: &str) -> bool {
    argon2.verify_encoded(hash, password.as_bytes()).unwrap_or(false);
}

struct Claims {
    sub: i32,
    role: String,
    exp: usize
}

pub fn create_jwt(user_id: i32, role: &str, secret: &str) -> String {
    let expiration = chrono::Utc::now()
    .checked_add_signed(chrono::duration::hours(24))
    .unwrap()
    .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        role: role,
        exp: expiration,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap();
}

pub fn verify_jwt


