use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: &'static str,
    pub version: &'static str,
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(HealthStatus {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check));
}
