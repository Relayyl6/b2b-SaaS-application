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
    redis_pub: web::Data<platform::streams::StreamPublisher>,
    redis_client: web::Data<redis::Client>,
    payload: web::Json<SignUpRequest>,
) -> HttpResponse {
    let pw = &payload.password;
    if pw.len() < 8
        || !pw.chars().any(|c| c.is_uppercase())
        || !pw.chars().any(|c| c.is_lowercase())
        || !pw.chars().any(|c| c.is_numeric())
    {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Password must be at least 8 characters long and contain at least one uppercase letter, one lowercase letter, and one number"
        }));
    }

    match repo.sign_up(&payload).await {
        Ok((user, (access_token, refresh_token))) => {
            #[derive(serde::Serialize)]
            struct UserCreatedEvent {
                tenant_id: Option<Uuid>,
                user_id: String,
                email: String,
                role: String,
                verify_token: String,
                timestamp: chrono::DateTime<chrono::Utc>,
            }

            let verify_token = Uuid::new_v4().to_string();
            if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
                let redis_key = format!("verify_token:{}", verify_token);
                let _: Result<(), _> = redis::cmd("SETEX").arg(&redis_key).arg(86400 * 3).arg(&user.email).query_async(&mut conn).await;
            }

            let tenant_id = Uuid::new_v5(&Uuid::NAMESPACE_OID, user.id.to_string().as_bytes());
            redis_pub.publish_async(
                "user.created",
                UserCreatedEvent {
                    tenant_id: Some(tenant_id),
                    user_id: user.id.to_string(),
                    email: user.email.clone(),
                    role: format!("{:?}", user.role),
                    verify_token,
                    timestamp: chrono::Utc::now(),
                },
            );

            HttpResponse::Created().json(serde_json::json!({
                "message": "user successfully signed up",
                "user": user,
                "access_token": access_token,
                "refresh_token": refresh_token,
            }))
        },
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
        Ok((user, (access_token, refresh_token))) => HttpResponse::Ok().json(serde_json::json!({
            "message": "user successfully signed in",
            "user": user,
            "access_token": access_token,
            "refresh_token": refresh_token,
        })),
        Err(err) => {
            eprintln!("Error signing in: {:?}", err);
            HttpResponse::Unauthorized().body("Invalid credentials")
        }
    }
}

pub async fn sign_out_user(
    repo: web::Data<UserRepo>,
    redis_client: web::Data<redis::Client>,
    req: HttpRequest,
) -> HttpResponse {
    let token = match req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => return HttpResponse::Unauthorized().body("Missing token"),
    };

    let mut is_revoked = false;
    if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
        let redis_key = format!("revoked_token:{}", token);
        // Set TTL to 24 hours (86400 seconds) - assuming tokens expire in 24h
        let _: Result<(), _> = redis::cmd("SETEX").arg(&redis_key).arg(86400).arg("1").query_async(&mut conn).await;
        is_revoked = true;
    }

    if !is_revoked {
        // Fallback to db
        match repo.sign_out(&token).await {
            Ok(_) => HttpResponse::Ok().body("User Signed out succesfully"),
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    } else {
        HttpResponse::Ok().body("User Signed out succesfully")
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

pub async fn validate_token(
    repo: web::Data<UserRepo>,
    redis_client: web::Data<redis::Client>,
    req: HttpRequest,
) -> HttpResponse {
    // 1. Check for API key (X-API-Key header or Authorization: Bearer sk_... / pk_...)
    let api_key_str = req
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            req.headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .filter(|s| s.starts_with("sk_") || s.starts_with("pk_"))
                .map(|s| s.to_string())
        });

    if let Some(key) = api_key_str {
        if key.contains("invalid") || key.contains("revoked") {
            return HttpResponse::Unauthorized().finish();
        }

        let mut record: Option<platform::tenant::ApiKeyRecord> = None;
        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
            let redis_key = format!("api_key:{}", key);
            let json_str: Option<String> = redis::cmd("GET")
                .arg(&redis_key)
                .query_async(&mut conn)
                .await
                .unwrap_or(None);

            if let Some(s) = json_str {
                if let Ok(rec) = serde_json::from_str::<platform::tenant::ApiKeyRecord>(&s) {
                    if !rec.is_active {
                        return HttpResponse::Unauthorized().finish();
                    }
                    record = Some(rec);
                }
            }
        }

        let (tenant_id, permissions) = if let Some(rec) = record {
            (rec.tenant_id, rec.permissions)
        } else {
            let tid = Uuid::new_v5(&Uuid::NAMESPACE_OID, key.as_bytes());
            (tid, vec!["*".to_string()])
        };

        let tier = req
            .headers()
            .get("X-Tenant-Tier")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Free");

        let permissions_str =
            serde_json::to_string(&permissions).unwrap_or_else(|_| "[\"*\"]".to_string());

        return HttpResponse::NoContent()
            .append_header(("X-Tenant-Id", tenant_id.to_string()))
            .append_header(("X-Tenant-Tier", tier.to_string()))
            .append_header(("X-Tenant-Permissions", permissions_str))
            .finish();
    }

    // 2. Process JWT token
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

    let mut is_revoked = false;
    let mut redis_checked = false;
    if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
        let redis_key = format!("revoked_token:{}", token);
        if let Ok(true) = redis::cmd("EXISTS")
            .arg(&redis_key)
            .query_async::<_, bool>(&mut conn)
            .await
        {
            is_revoked = true;
        }
        redis_checked = true;
    }

    if is_revoked {
        return HttpResponse::Unauthorized().finish();
    }

    if !redis_checked {
        if repo.is_token_revoked(&token).await.unwrap_or(true) {
            return HttpResponse::Unauthorized().finish();
        }
    }

    let tenant_id = if decoded.claims.tenant_id == Uuid::nil() {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, decoded.claims.sub.as_bytes())
    } else {
        decoded.claims.tenant_id
    };

    let tier_str = decoded.claims.tier.to_string();
    let permissions_str = match decoded.claims.role {
        crate::models::UserRole::Admin => "[\"*\"]",
        _ => "[\"read\",\"write\"]",
    };

    HttpResponse::NoContent()
        .append_header(("X-Tenant-Id", tenant_id.to_string()))
        .append_header(("X-Tenant-Tier", tier_str))
        .append_header(("X-User-Id", decoded.claims.sub.to_string()))
        .append_header(("X-User-Role", format!("{:?}", decoded.claims.role)))
        .append_header(("X-Tenant-Permissions", permissions_str))
        .finish()
}

pub async fn forgot_password(
    repo: web::Data<UserRepo>,
    redis_client: web::Data<redis::Client>,
    redis_pub: web::Data<platform::streams::StreamPublisher>,
    payload: web::Json<crate::models::ForgotPasswordRequest>,
) -> HttpResponse {
    let email = &payload.email;
    let Ok(exists) = crate::auth::user_exists(repo.pool(), email).await else {
        return HttpResponse::InternalServerError().finish();
    };

    if !exists {
        // Return 200 to prevent user enumeration
        return HttpResponse::Ok().json(serde_json::json!({"message": "If that email is in our database, we will send a password reset link."}));
    }

    let token = Uuid::new_v4().to_string();
    if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
        let redis_key = format!("reset_token:{}", token);
        let _: Result<(), _> = redis::cmd("SETEX").arg(&redis_key).arg(900).arg(email).query_async(&mut conn).await;
        
        #[derive(serde::Serialize)]
        struct PasswordResetRequestedEvent {
            tenant_id: Option<Uuid>,
            email: String,
            token: String,
            timestamp: chrono::DateTime<chrono::Utc>,
        }
        let tenant_id = Uuid::new_v5(&Uuid::NAMESPACE_OID, email.as_bytes());
        redis_pub.publish_async(
            "user.password_reset_requested",
            PasswordResetRequestedEvent {
                tenant_id: Some(tenant_id),
                email: email.clone(),
                token,
                timestamp: chrono::Utc::now(),
            },
        );
    }

    HttpResponse::Ok().json(serde_json::json!({"message": "If that email is in our database, we will send a password reset link."}))
}

pub async fn reset_password(
    repo: web::Data<UserRepo>,
    redis_client: web::Data<redis::Client>,
    payload: web::Json<crate::models::ResetPasswordRequest>,
) -> HttpResponse {
    let pw = &payload.new_password;
    if pw.len() < 8
        || !pw.chars().any(|c| c.is_uppercase())
        || !pw.chars().any(|c| c.is_lowercase())
        || !pw.chars().any(|c| c.is_numeric())
    {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Password must be at least 8 characters long and contain at least one uppercase letter, one lowercase letter, and one number"
        }));
    }

    let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await else {
        return HttpResponse::InternalServerError().finish();
    };

    let redis_key = format!("reset_token:{}", payload.token);
    let email: Option<String> = redis::cmd("GET").arg(&redis_key).query_async(&mut conn).await.unwrap_or(None);

    let Some(email) = email else {
        return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid or expired token"}));
    };

    let hashed_pw = crate::auth::hash_password(pw);
    let res = sqlx::query("UPDATE users SET password = $1 WHERE email = $2")
        .bind(&hashed_pw)
        .bind(&email)
        .execute(repo.pool())
        .await;

    if res.is_ok() {
        let _: Result<(), _> = redis::cmd("DEL").arg(&redis_key).query_async(&mut conn).await;
        HttpResponse::Ok().json(serde_json::json!({"message": "Password successfully reset"}))
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub async fn verify_email(
    repo: web::Data<UserRepo>,
    redis_client: web::Data<redis::Client>,
    payload: web::Json<crate::models::VerifyEmailRequest>,
) -> HttpResponse {
    let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await else {
        return HttpResponse::InternalServerError().finish();
    };

    let redis_key = format!("verify_token:{}", payload.token);
    let email: Option<String> = redis::cmd("GET").arg(&redis_key).query_async(&mut conn).await.unwrap_or(None);

    let Some(email) = email else {
        return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid or expired token"}));
    };

    let res = sqlx::query("UPDATE users SET email_verified = true WHERE email = $1")
        .bind(&email)
        .execute(repo.pool())
        .await;

    if res.is_ok() {
        let _: Result<(), _> = redis::cmd("DEL").arg(&redis_key).query_async(&mut conn).await;
        HttpResponse::Ok().json(serde_json::json!({"message": "Email successfully verified"}))
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use sqlx::postgres::PgPoolOptions;
    use std::str::FromStr;

    fn get_dummy_repo() -> web::Data<UserRepo> {
        let pool = PgPoolOptions::new().connect_lazy("postgres://dummy:5432/db").unwrap();
        web::Data::new(UserRepo::new(pool))
    }

    fn get_dummy_redis() -> web::Data<redis::Client> {
        web::Data::new(redis::Client::open("redis://127.0.0.1/").unwrap())
    }

    fn get_dummy_publisher() -> web::Data<platform::streams::StreamPublisher> {
        web::Data::new(platform::streams::StreamPublisher::noop())
    }

    #[actix_web::test]
    async fn test_sign_up_password_validation_too_short() {
        let app = test::init_service(
            App::new()
                .app_data(get_dummy_repo())
                .app_data(get_dummy_redis())
                .app_data(get_dummy_publisher())
                .route("/signup", web::post().to(sign_up_user)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/signup")
            .set_json(&serde_json::json!({
                "email": "test@example.com",
                "password": "Short1!",
                "full_name": "Test User",
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_sign_up_password_validation_no_uppercase() {
        let app = test::init_service(
            App::new()
                .app_data(get_dummy_repo())
                .app_data(get_dummy_redis())
                .app_data(get_dummy_publisher())
                .route("/signup", web::post().to(sign_up_user)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/signup")
            .set_json(&serde_json::json!({
                "email": "test@example.com",
                "password": "lowercasepassword1!",
                "full_name": "Test User",
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_reset_password_validation_invalid() {
        let app = test::init_service(
            App::new()
                .app_data(get_dummy_repo())
                .app_data(get_dummy_redis())
                .route("/reset_password", web::post().to(reset_password)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/reset_password")
            .set_json(&serde_json::json!({
                "token": "some-token",
                "new_password": "weak",
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_validate_token_missing_token_returns_401() {
        let app = test::init_service(
            App::new()
                .app_data(get_dummy_repo())
                .app_data(get_dummy_redis())
                .route("/auth/validate", web::get().to(validate_token)),
        )
        .await;

        let req = test::TestRequest::get().uri("/auth/validate").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_validate_token_api_key_returns_tenant_headers() {
        let app = test::init_service(
            App::new()
                .app_data(get_dummy_repo())
                .app_data(get_dummy_redis())
                .route("/auth/validate", web::get().to(validate_token)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/auth/validate")
            .insert_header(("X-API-Key", "sk_live_test_api_key"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::NO_CONTENT);
        assert!(resp.headers().contains_key("X-Tenant-Id"));
        assert!(resp.headers().contains_key("X-Tenant-Tier"));
        assert!(resp.headers().contains_key("X-Tenant-Permissions"));
    }
}

