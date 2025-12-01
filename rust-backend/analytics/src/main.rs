mod events;
mod publisher;
mod analytics_worker;
mod dashboard;

use tracing_subscriber::FmtSubscriber;
use tokio::task;

// Your analytics service might consume events like:
//          InventoryViewed
//          ProductClicked
//          OrderInitiated
//          OrderCompleted
//          PaymentProcessed

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing
    let subscriber = FmtSubscriber::builder().with_env_filter("info").finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // choose role: worker, publisher sample, dashboard. For demo run worker + dashboard.
    let worker_task = task::spawn(async {
        if let Err(e) = analytics_worker::run_worker().await {
            tracing::error!("Worker error: {:?}", e);
        }
    });

    let dashboard_task = task::spawn(async {
        if let Err(e) = dashboard::run_dashboard().await {
            tracing::error!("Dashboard error: {:?}", e);
        }
    });

    tokio::try_join!(worker_task, dashboard_task)?;
    Ok(())
}
