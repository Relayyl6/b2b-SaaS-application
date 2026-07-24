// use crate::db::{sign_in, sign_out, sign_up, update_user, delete_user};
use actix_web::{HttpRequest, HttpResponse, web};
use jsonwebtoken::{DecodingKey, Validation, decode};
// use sqlx::PgPool;
use crate::models::{SignInRequest, SignUpRequest};
// use crate::auth::{hash_password, verify_password, create_jwt, verify_jwt, user_exists};
// use std::env;
use crate::db::UserRepo;
use serde_json;
use std::env;
use uuid::Uuid;

// Handler portion
pub async fn sign_up_user(
    repo: web::Data<UserRepo>,
    payload: web::Json<SignUpRequest>,
) -> HttpResponse {
    match repo.sign_up(&payload).await {
        Ok((user, token)) => HttpResponse::Created().json(serde_json::json!({
            "message": "user successfully signed up",
            "user": user,
            "token": token,
        })),
        Err(err) => {
            eprintln!("Error registering user: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn sign_in_user(
    repo: web::Data<UserRepo>,
    payload: web::Json<SignInRequest>,
) -> HttpResponse {
    match repo.sign_in(&payload).await {
        Ok((user, token)) => HttpResponse::Ok().json(serde_json::json!({
            "message": "user successfully signed in",
            "user": user,
            "token": token,
        })),
        Err(err) => {
            eprintln!("Error signing in: {:?}", err);
            HttpResponse::Unauthorized().body("Invalid credentials")
        }
    }
}

pub async fn sign_out_user(repo: web::Data<UserRepo>, req: HttpRequest) -> HttpResponse {
    let token = match req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => return HttpResponse::Unauthorized().body("Missing token"),
    };

    match repo.sign_out(&token).await {
        Ok(_) => HttpResponse::Ok().body("User Signed out succesfully"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_user(repo: web::Data<UserRepo>, path: web::Path<Uuid>) -> HttpResponse {
    let user_id = path.into_inner();
    match repo.get_user_details(user_id).await {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Not found"),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            HttpResponse::InternalServerError().body("DB error")
        }
    }
}

pub async fn validate_token(repo: web::Data<UserRepo>, req: HttpRequest) -> HttpResponse {
    let token = match req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => return HttpResponse::Unauthorized().finish(),
    };

    let secret = env::var("SECRET").unwrap_or_else(|_| "something".to_string());
    let decoded = match decode::<crate::models::Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(decoded) => decoded,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };

    if repo.is_token_revoked(&token).await.unwrap_or(true) {
        return HttpResponse::Unauthorized().finish();
    }

    HttpResponse::NoContent()
        .append_header(("X-User-Id", decoded.claims.sub.to_string()))
        .append_header(("X-User-Role", format!("{:?}", decoded.claims.role)))
        .finish()
}
