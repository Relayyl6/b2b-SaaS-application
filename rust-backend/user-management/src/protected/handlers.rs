use crate::db::{sign_in, sign_out, sign_up, update_user, delete_user};
use actix_web::{web, HttpResponse, Responder, HttpRequest};
use sqlx::PgPool;
use crate::models::{Users, SignUpRequest, SignInRequest, SignOutRequest, AuthResponse, UpdateUserRequest, DeleteUserRequest, UserRole};
use crate::auth::{hash_password, verify_password, create_jwt, verify_jwt, user_exists};
use std::env;
use uuid::Uuid;


// Handler portion
pub async fn sign_up_user(
    repo: web::Data::<UserRepo>,
    payload: web::json<SignUpRequest>,
) -> HttpResponse {
    match repo.sign_up(&payload).await {
        Ok(user) => HttpResponse::Created().json(user),
        Err(err) => {
            eprintln!("Error registering user: {:?}", err),
            HttpResponse::InternalServerError().body("Failed to register user")
        }
    }
}

pub async fn sign_in_user(
    repo: web::Data::<UserRepo>,
    payload: web::json::<SignInRequest>,
) -> HttpResponse {
    match repo.sign_in(&payload).await {
        Ok(user) => HttpResponse::Created.json(user),
        Err(err) => {
            eprintln!("Error signing in: {:?}", err),
            HttpResponse::InternalServerError().body("Failed to sign in user")
        }
    }
}

pub async fn sign_out_user

// yesterday you were using &email, i understad that was necessary bcuase email is a string ype that is heap allocated type and would need to be referenced but then you sometimes used .as_ref() and now you are using .as_deref(), why are they different 

pub async fn update_user_handler(
    req: HttpRequest,
    repo: web::Data<UserRepo>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateUserRequest>,
) -> HttpResponse {
    let auth_user = match req.extensions().get::<Users>() {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = path.into_inner();

    // Optional: check if user has permission (e.g., admin or self)
    if auth_user.id != user_id && auth_user.role != "Admin" {
        return HttpResponse::Forbidden().body("Not authorized");
    }

    match update_user(user_id, &payload).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

pub async fn delete_user_handler(
    req: HttpRequest,
    repo: web::Data<UserRepo>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth_user = match req.extensions().get::<Users>() {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let target_user_id = path.into_inner();

    // Only admin or the user themselves can delete
    if auth_user.id != target_user_id && auth_user.role != "Admin" {
        return HttpResponse::Forbidden().body("Not authorized");
    }

    match crate::db::delete_user(target_user_id).await {
        Ok(_) => HttpResponse::Ok().body("User deleted"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}