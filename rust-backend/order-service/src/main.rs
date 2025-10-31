use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use std::env;

mod db;
mod models;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("{}:{}", host, port);

    let pool = db::get_db_pool().await;

    println!("🚀 Order Service running at http://{}", addr);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(routes::create_order)
            .service(routes::get_order)
            .service(routes::update_status)
    })
    .bind(addr)?
    .run()
    .await
}
