use crate::models::{
    CreateNotificationRequest, ListNotificationsQuery, Notification, NotificationDevice,
    RegisterDeviceRequest,
};
use chrono::Utc;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationRepo {
    pool: sqlx::PgPool,
}

impl NotificationRepo {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        req: &CreateNotificationRequest,
    ) -> Result<Notification, sqlx::Error> {
        let event_type = req
            .event_type
            .clone()
            .unwrap_or_else(|| "notification.manual".to_string());
        let priority = req
            .priority
            .clone()
            .unwrap_or(crate::models::NotificationPriority::Normal);
        let recipient = req
            .recipient
            .clone()
            .or_else(|| req.user_id.map(|id| format!("user:{id}")))
            .or_else(|| req.supplier_id.map(|id| format!("supplier:{id}")))
            .unwrap_or_else(|| "system".to_string());

        sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (
                user_id, supplier_id, order_id, event_type, channel, priority,
                recipient, subject, body, payload
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, COALESCE($10, '{}'::jsonb))
            RETURNING *
            "#,
        )
        .bind(req.user_id)
        .bind(req.supplier_id)
        .bind(req.order_id)
        .bind(event_type)
        .bind(&req.channel)
        .bind(priority)
        .bind(recipient)
        .bind(&req.subject)
        .bind(&req.body)
        .bind(req.payload.as_ref())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list(
        &self,
        query: &ListNotificationsQuery,
    ) -> Result<Vec<Notification>, sqlx::Error> {
        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        let offset = query.offset.unwrap_or(0).max(0);

        let mut builder = QueryBuilder::<Postgres>::new("SELECT * FROM notifications WHERE true");

        if let Some(user_id) = query.user_id {
            builder.push(" AND user_id = ");
            builder.push_bind(user_id);
        }

        if let Some(supplier_id) = query.supplier_id {
            builder.push(" AND supplier_id = ");
            builder.push_bind(supplier_id);
        }

        if let Some(status) = &query.status {
            builder.push(" AND status = ");
            builder.push_bind(status);
        }

        builder.push(" ORDER BY created_at DESC LIMIT ");
        builder.push_bind(limit);
        builder.push(" OFFSET ");
        builder.push_bind(offset);

        builder.build_query_as().fetch_all(&self.pool).await
    }

    pub async fn get(&self, id: Uuid) -> Result<Notification, sqlx::Error> {
        sqlx::query_as::<_, Notification>("SELECT * FROM notifications WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn mark_sent(&self, id: Uuid) -> Result<Notification, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            r#"
            UPDATE notifications
            SET status = 'sent', attempts = attempts + 1, sent_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn mark_failed(&self, id: Uuid, error: &str) -> Result<Notification, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            r#"
            UPDATE notifications
            SET status = 'failed', attempts = attempts + 1, last_error = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(error)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn mark_read(&self, id: Uuid) -> Result<Notification, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            r#"
            UPDATE notifications
            SET status = 'read', read_at = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn pending_batch(&self, limit: i64) -> Result<Vec<Notification>, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            r#"
            SELECT *
            FROM notifications
            WHERE status = 'pending'
              AND attempts < 5
            ORDER BY
              CASE priority
                WHEN 'critical' THEN 0
                WHEN 'high' THEN 1
                WHEN 'normal' THEN 2
                ELSE 3
              END,
              created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await
    }

    pub async fn register_device(
        &self,
        req: &RegisterDeviceRequest,
    ) -> Result<NotificationDevice, sqlx::Error> {
        sqlx::query_as::<_, NotificationDevice>(
            r#"
            INSERT INTO notification_devices (
                user_id, platform, push_token, provider, device_id, app_version
            )
            VALUES ($1, $2, $3, COALESCE($4, 'expo'), $5, $6)
            ON CONFLICT(push_token) DO UPDATE SET
                user_id = EXCLUDED.user_id,
                platform = EXCLUDED.platform,
                provider = EXCLUDED.provider,
                device_id = EXCLUDED.device_id,
                app_version = EXCLUDED.app_version,
                enabled = TRUE,
                last_seen_at = NOW(),
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(req.user_id)
        .bind(&req.platform)
        .bind(&req.push_token)
        .bind(&req.provider)
        .bind(&req.device_id)
        .bind(&req.app_version)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_user_devices(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<NotificationDevice>, sqlx::Error> {
        sqlx::query_as::<_, NotificationDevice>(
            r#"
            SELECT *
            FROM notification_devices
            WHERE user_id = $1 AND enabled = TRUE
            ORDER BY last_seen_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn disable_device(&self, id: Uuid) -> Result<NotificationDevice, sqlx::Error> {
        sqlx::query_as::<_, NotificationDevice>(
            r#"
            UPDATE notification_devices
            SET enabled = FALSE, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }
}
