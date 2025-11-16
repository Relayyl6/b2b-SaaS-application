// use crate::db::{sign_in, sign_out, sign_up, update_user, delete_user};
use actix_web::{web, HttpResponse, HttpRequest};
// use sqlx::PgPool;
use crate::models::{SignUpRequest, SignInRequest};
// use crate::auth::{hash_password, verify_password, create_jwt, verify_jwt, user_exists};
// use std::env;
// use uuid::Uuid;
use crate::db::UserRepo;


// Handler portion
pub async fn sign_up_user(
    repo: web::Data<UserRepo>,
    payload: web::Json<SignUpRequest>,
) -> HttpResponse {
    match repo.sign_up(&payload).await {
        Ok((user, token)) => HttpResponse::Created().json((user, token)),
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
        Ok((user, token)) => HttpResponse::Ok().json((user, token)),
        Err(err) => {
            eprintln!("Error signing in: {:?}", err);
            HttpResponse::Unauthorized().body("Invalid credentials")
        }
    }
}


pub async fn sign_out_user(
    repo: web::Data<UserRepo>,
    req: HttpRequest,
) -> HttpResponse {
    let token = match req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => return HttpResponse::Unauthorized().body("Missing token"),
    };

    match repo.sign_out(&token).await {
        Ok(_) => HttpResponse::Ok().body("Signed out"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

