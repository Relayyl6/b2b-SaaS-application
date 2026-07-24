mod events;
mod handlers;
mod models;
mod publisher;
mod tests;
mod worker;

use crate::handlers::AnalyticsRepo;
use crate::worker::consumer::RabbitConsumer;
use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use redis::Client;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio;
use tokio::spawn;
use tracing::{error, subscriber};
use tracing_subscriber::FmtSubscriber;

// The analytics service might consume events like:
//          InventoryViewed
//          ProductClicked
//          OrderInitiated
//          OrderCompleted
//          PaymentProcessed

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    // tracing
    let subscriber = FmtSubscriber::builder().with_env_filter("info").finish();
    let _ = subscriber::set_global_default(subscriber);

    let port = env::var("SERVICE_PORT").unwrap_or_else(|_| "3007".to_string());
    let db_url = env::var("DATABASE_URL").expect("Database url not set");
    let redis_url = env::var("REDIS_URL");

    // Redis client
    let redis_client = web::Data::new(
        redis_url
            .as_ref()
            .map(|url| Client::open(url.as_str()))
            .unwrap_or_else(|_| {
                eprintln!("⚠️ REDIS_URL not set — using noop client.");
                Ok(Client::open("redis://localhost:6379").unwrap())
            })
            .unwrap(),
    );

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("postgres");
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("❌ Migration failed: {:?}", e);
        std::process::exit(1);
    };

    let repo = web::Data::new(AnalyticsRepo::new(&pool));

    let rabbitconsume = web::Data::new(RabbitConsumer::new(&pool));
    let consumer = rabbitconsume.clone();

    // choose role: worker, publisher sample, dashboard. For demo run worker + dashboard.
    let pool_clone = pool.clone();
    let redis_client_clone = redis_client.clone();
    spawn(async move {
        if let Err(e) = consumer.run(&pool_clone, &redis_client_clone).await {
            error!("Worker error: {:?}", e);
        }
    });

    tracing::info!("Analytics Service listening on 0.0.0.0:{}", port);

    let _ = HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .app_data(repo.clone())
            .app_data(rabbitconsume.clone())
            .route(
                "/analytics",
                web::post().to(handlers::AnalyticsRepo::analytics_handler),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await;

    Ok(())
}
