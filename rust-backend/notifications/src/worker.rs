use actix_web::web;
use std::time::Duration;

use crate::db::NotificationRepo;
use crate::provider::NotificationProvider;

pub async fn start_delivery_worker(
    repo: web::Data<NotificationRepo>,
    provider: web::Data<NotificationProvider>,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            let pending = match repo.pending_batch(25).await {
                Ok(batch) => batch,
                Err(e) => {
                    eprintln!("notification worker query failed: {e}");
                    continue;
                }
            };

            for notification in pending {
                let id = notification.id;
                match provider.send(&notification).await {
                    Ok(()) => {
                        if let Err(e) = repo.mark_sent(id).await {
                            eprintln!("failed marking notification sent: {e}");
                        }
                    }
                    Err(error) => {
                        if let Err(e) = repo.mark_failed(id, &error).await {
                            eprintln!("failed marking notification failed: {e}");
                        }
                    }
                }
            }
        }
    });
}
