// use crate::db::{sign_in, sign_out, sign_up, update_user, delete_user};
use actix_web::{web, HttpResponse, HttpRequest};
use actix_web::HttpMessage;
// use sqlx::PgPool;
use crate::models::{Users, UpdateUserRequest};
// use crate::auth::{hash_password, verify_password, create_jwt, verify_jwt, user_exists};
// use std::env;
use uuid::Uuid;
use crate::db::UserRepo;
// use actix_web::
use crate::models::UserRole;

pub async fn update_user_handler(
    req: HttpRequest,
    repo: web::Data<UserRepo>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateUserRequest>,
) -> HttpResponse {
    let extensions = req.extensions();
    let auth_user = match extensions.get::<Users>() {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let user_id = path.into_inner();

    // Optional: check if user has permission (e.g., admin or self)
    if auth_user.id != user_id && auth_user.role == UserRole::Admin  {
        return HttpResponse::Forbidden().body("Not authorized");
    }

    match repo.update_user(user_id, &payload).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

pub async fn delete_user_handler(
    req: HttpRequest,
    repo: web::Data<UserRepo>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let extensions = req.extensions();
    let auth_user = match extensions.get::<Users>() {
        Some(u) => u,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let target_user_id = path.into_inner();

    // Only admin or the user themselves can delete
    if auth_user.id != target_user_id && auth_user.role == UserRole::Admin {
        return HttpResponse::Forbidden().body("Not authorized");
    }

    match repo.delete_user(target_user_id).await {
        Ok(_) => HttpResponse::Ok().body("User deleted"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}