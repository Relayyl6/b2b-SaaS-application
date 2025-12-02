mod events;
mod dashboard;
mod worker;
mod handler;
mod publisher;

use crate::worker::consumer as consumer
use tokio:spawn;
use tracing_subscriber::FmtSubscriber;
use tokio;
use sqlx::postgres::PgPoolOptions;
use crate::handler::AnalyticsRepo;

// The analytics service might consume events like:
//          InventoryViewed
//          ProductClicked
//          OrderInitiated
//          OrderCompleted
//          PaymentProcessed

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // tracing
    let subscriber = FmtSubscriber::builder().with_env_filter("info").finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let port = env::var("SERVICE_PORT").unwrap_or_else(|_| "3002".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("postgres");
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("❌ Migration failed: {:?}", e);
        std::process::exit(1);
    }

    let repo = web::Data::new(AnalyticsRepo::new(&pool));

    // choose role: worker, publisher sample, dashboard. For demo run worker + dashboard.
    let _ = spawn(async {
        if let Err(e) = consumer::run_worker(&pool).await {
            tracing::error!("Worker error: {:?}", e);
        }
    });

    println!("Analytics Service running on htts://localshost: port")

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .app_data(repo.clone())
            .app_data(redis_client.clone())
            .route("/analytics", web::get().to(analytics_handler::analytics_handler))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await

    Ok(())
}
