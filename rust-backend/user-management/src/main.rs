mod auth;
mod db;
mod middleware;
mod models;
mod redis_pub;
// mod redis_sub;

mod protected;
mod unprotected;

use crate::db::UserRepo;
use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use redis::Client as RedisClient;
use sqlx::PgPool;
use std::env;

use crate::protected::handlers as protected_handlers;
use crate::unprotected::handlers as unprotected_handlers;
use platform::{metrics, observability};

use platform::middleware::tenant_middleware::TenantAuthMiddleware;
use platform::db_router::DynamicPoolRouter;
// use protected::handlers;

// use crate::unhandlers::{sign_up_user, sign_in_user, sign_out_user}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    observability::init_observability("user-management");
    metrics::init_metrics("user-management");

    let database_url =
        env::var("DATABASE_URL").expect("Database url must be set in the environment variable");
    let redis_url = env::var("REDIS_URL").ok();
    let port = env::var("SERVICE_PORT")
        .or_else(|_| env::var("PORT"))
        .unwrap_or_else(|_| "3004".to_string());
    let jwt_secret = env::var("SECRET").unwrap_or_else(|_| "something".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to postgres database");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migrations Failed");

    let repo = web::Data::new(UserRepo::new(pool.clone()));

    let db_router = web::Data::new(DynamicPoolRouter::new(pool.clone()));
    let redis_pub = match &redis_url {
        Some(url) => match platform::streams::StreamPublisher::new(url) {
            Ok(pubw) => web::Data::new(pubw),
            Err(e) => {
                eprintln!("⚠️ Failed to connect to Redis: {:?}", e);
                eprintln!("⚠️ Continuing without Redis publishing capabilities...");
                web::Data::new(platform::streams::StreamPublisher::noop())
            }
        },
        None => {
            eprintln!("⚠️ No REDIS_URL configured — using no-op publisher");
            web::Data::new(platform::streams::StreamPublisher::noop())
        }
    };

    let redis_url_str = redis_url.clone().unwrap_or_else(|| "redis://127.0.0.1:6379".to_string());
    let redis_client_inner = RedisClient::open(redis_url_str).expect("redis client");
    let redis_client = web::Data::new(redis_client_inner.clone());

    let middleware = TenantAuthMiddleware::with_redis(redis_client_inner.clone())
        .with_secret(jwt_secret.clone());
    tracing::info!("User Management Service listening on 0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(repo.clone())
            .app_data(redis_pub.clone())
            .app_data(redis_client.clone())
            .app_data(db_router.clone())
            .service(
                web::scope("/protected") // all /protected/* routes
                    .wrap(middleware.clone()) // middleware only applies here
                    .route(
                        "/update/{id}",
                        web::put().to(protected_handlers::update_user_handler),
                    )
                    .route(
                        "/delete/{id}",
                        web::delete().to(protected_handlers::delete_user_handler),
                    ),
            )
            .service(
                web::scope("/admin")
                    .wrap(middleware::rbac::RequireRole::new(vec![models::UserRole::Admin]))
                    .wrap(middleware.clone())
                    .route(
                        "/stats",
                        web::get().to(protected_handlers::admin_stats_handler),
                    ),
            )
            // other unprotected routes outside the scope
            .route(
                "/signup",
                web::post().to(unprotected_handlers::sign_up_user),
            )
            .route(
                "/signin",
                web::post().to(unprotected_handlers::sign_in_user),
            )
            .route(
                "/signout",
                web::post().to(unprotected_handlers::sign_out_user),
            )
            .route(
                "/get_user/{id}",
                web::get().to(unprotected_handlers::get_user),
            )
            .route(
                "/auth/validate",
                web::get().to(unprotected_handlers::validate_token),
            )
            .route(
                "/forgot-password",
                web::post().to(unprotected_handlers::forgot_password),
            )
            .route(
                "/reset-password",
                web::post().to(unprotected_handlers::reset_password),
            )
            .route(
                "/verify-email",
                web::post().to(unprotected_handlers::verify_email),
            )
            .route("/metrics", web::get().to(metrics::metrics_handler))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
