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

    // Only admin or the user themselves can update
    if auth_user.id != user_id && auth_user.role != UserRole::Admin {
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
    if auth_user.id != target_user_id && auth_user.role != UserRole::Admin {
        return HttpResponse::Forbidden().body("Not authorized");
    }

    match repo.delete_user(target_user_id).await {
        Ok(_) => HttpResponse::Ok().body("User deleted successully"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

pub async fn admin_stats_handler() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Admin stats retrieved successfully",
        "total_users": 42
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_admin_stats_handler() {
        let app = test::init_service(App::new().route("/stats", web::get().to(admin_stats_handler))).await;
        let req = test::TestRequest::get().uri("/stats").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["message"], "Admin stats retrieved successfully");
        assert_eq!(body["total_users"], 42);
    }
}
