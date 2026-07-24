use crate::db::UserRepo;
use crate::models::UserRole;
use crate::models::{UpdateUserRequest, Users};
use actix_web::HttpMessage;
use actix_web::{HttpRequest, HttpResponse, web};
use serde_json;
use uuid::Uuid;

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
    if auth_user.id != user_id && auth_user.role == UserRole::Admin {
        return HttpResponse::Forbidden().body("Not authorized");
    }

    match repo.update_user(user_id, &payload).await {
        Ok(user) => HttpResponse::Ok().json(serde_json::json!({
            "message": "user updated successfully",
            "user": user,
        })),
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
        Ok(_) => HttpResponse::Ok().body("User deleted successully"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
