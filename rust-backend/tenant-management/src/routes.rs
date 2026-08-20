use actix_web::web;
use crate::handlers::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/v1/tenants", web::post().to(create_tenant))
       .route("/v1/tenants/keys", web::post().to(generate_api_key_handler));
}
