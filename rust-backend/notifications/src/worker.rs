use actix_web::web;
use std::time::Duration;

use crate::db::NotificationRepo;
use crate::dlq_pub::DlqPublisher;
use crate::provider::NotificationProvider;

pub async fn start_delivery_worker(
    pool: sqlx::PgPool,
    provider: web::Data<NotificationProvider>,
    dlq_publisher: web::Data<DlqPublisher>,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            interval.tick().await;

            let mut conn = match pool.acquire().await {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("notification worker failed to acquire connection: {e}");
                    continue;
                }
            };

            let pending = match NotificationRepo::pending_batch(&mut conn, 25).await {
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
                        if let Err(e) = NotificationRepo::mark_sent(&mut conn, id).await {
                            eprintln!("failed marking notification sent: {e}");
                        }
                    }
                    Err(error) => {
                        if let Err(e) = NotificationRepo::mark_failed(&mut conn, id, &error).await {
                            eprintln!("failed marking notification failed: {e}");
                        }
                        dlq_publisher.publish_to_dlq(&notification, &error).await;
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::web;
    use crate::dlq_pub::DlqPublisher;
    use crate::provider::NotificationProvider;

    #[sqlx::test]
    #[ignore]
    async fn test_worker_pushes_to_dlq_on_failure(pool: sqlx::PgPool) {
        let provider = web::Data::new(NotificationProvider::from_env());
        let dlq = web::Data::new(DlqPublisher::new().await);
        start_delivery_worker(pool, provider, dlq).await;
    }
}
