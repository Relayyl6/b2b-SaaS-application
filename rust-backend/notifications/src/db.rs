use crate::models::{
    CreateNotificationRequest, ListNotificationsQuery, Notification, NotificationChannel,
    NotificationDevice, RegisterDeviceRequest, UpdatePreferencesRequest, UserPreference,
};
use chrono::Utc;
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationRepo;

impl NotificationRepo {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    pub async fn create(
        conn: &mut sqlx::PgConnection,
        tenant_id: Uuid,
        req: &CreateNotificationRequest,
    ) -> Result<Notification, sqlx::Error> {
        if let Some(user_id) = req.user_id {
            if let Ok(prefs) = Self::get_preferences(&mut *conn, tenant_id, user_id).await {
                let is_enabled = match req.channel {
                    NotificationChannel::Email => prefs.email_enabled,
                    NotificationChannel::Sms => prefs.sms_enabled,
                    NotificationChannel::Push => prefs.push_enabled,
                    NotificationChannel::InApp => prefs.in_app_enabled,
                };

                if !is_enabled {
                    return Err(sqlx::Error::Protocol(
                        "User opted out of this channel".to_string().into(),
                    ));
                }
            }
        }

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
                tenant_id, user_id, supplier_id, order_id, event_type, channel, priority,
                recipient, subject, body, payload
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, COALESCE($11, '{}'::jsonb))
            RETURNING *
            "#,
        )
        .bind(tenant_id)
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
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn list(
        conn: &mut sqlx::PgConnection,
        tenant_id: Uuid,
        query: &ListNotificationsQuery,
    ) -> Result<Vec<Notification>, sqlx::Error> {
        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        let offset = query.offset.unwrap_or(0).max(0);

        let mut builder = QueryBuilder::<Postgres>::new("SELECT * FROM notifications WHERE tenant_id = ");
        builder.push_bind(tenant_id);

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

        builder.build_query_as().fetch_all(&mut *conn).await
    }

    pub async fn get(
        conn: &mut sqlx::PgConnection,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Notification, sqlx::Error> {
        sqlx::query_as::<_, Notification>("SELECT * FROM notifications WHERE id = $1 AND tenant_id = $2")
            .bind(id)
            .bind(tenant_id)
            .fetch_one(&mut *conn)
            .await
    }

    pub async fn mark_sent(
        conn: &mut sqlx::PgConnection,
        id: Uuid,
    ) -> Result<Notification, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            r#"
            UPDATE notifications
            SET status = 'sent', attempts = attempts + 1, sent_at = NOW(), updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn mark_failed(
        conn: &mut sqlx::PgConnection,
        id: Uuid,
        error: &str,
    ) -> Result<Notification, sqlx::Error> {
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
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn mark_read(
        conn: &mut sqlx::PgConnection,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Notification, sqlx::Error> {
        sqlx::query_as::<_, Notification>(
            r#"
            UPDATE notifications
            SET status = 'read', read_at = $2, updated_at = NOW()
            WHERE id = $1 AND tenant_id = $3
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(Utc::now())
        .bind(tenant_id)
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn pending_batch(
        conn: &mut sqlx::PgConnection,
        limit: i64,
    ) -> Result<Vec<Notification>, sqlx::Error> {
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
        .fetch_all(&mut *conn)
        .await
    }

    pub async fn register_device(
        conn: &mut sqlx::PgConnection,
        tenant_id: Uuid,
        req: &RegisterDeviceRequest,
    ) -> Result<NotificationDevice, sqlx::Error> {
        sqlx::query_as::<_, NotificationDevice>(
            r#"
            INSERT INTO notification_devices (
                tenant_id, user_id, platform, push_token, provider, device_id, app_version
            )
            VALUES ($1, $2, $3, $4, COALESCE($5, 'expo'), $6, $7)
            ON CONFLICT(push_token) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
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
        .bind(tenant_id)
        .bind(req.user_id)
        .bind(&req.platform)
        .bind(&req.push_token)
        .bind(&req.provider)
        .bind(&req.device_id)
        .bind(&req.app_version)
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn list_user_devices(
        conn: &mut sqlx::PgConnection,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<NotificationDevice>, sqlx::Error> {
        sqlx::query_as::<_, NotificationDevice>(
            r#"
            SELECT *
            FROM notification_devices
            WHERE tenant_id = $1 AND user_id = $2 AND enabled = TRUE
            ORDER BY last_seen_at DESC
            "#,
        )
        .bind(tenant_id)
        .bind(user_id)
        .fetch_all(&mut *conn)
        .await
    }

    pub async fn disable_device(
        conn: &mut sqlx::PgConnection,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<NotificationDevice, sqlx::Error> {
        sqlx::query_as::<_, NotificationDevice>(
            r#"
            UPDATE notification_devices
            SET enabled = FALSE, updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn get_preferences(
        conn: &mut sqlx::PgConnection,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<UserPreference, sqlx::Error> {
        sqlx::query_as::<_, UserPreference>(
            r#"
            SELECT * FROM notification_preferences WHERE tenant_id = $1 AND user_id = $2
            "#,
        )
        .bind(tenant_id)
        .bind(user_id)
        .fetch_optional(&mut *conn)
        .await
        .map(|opt| {
            opt.unwrap_or_else(|| UserPreference {
                user_id,
                tenant_id,
                email_enabled: true,
                sms_enabled: true,
                push_enabled: true,
                in_app_enabled: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        })
    }

    pub async fn update_preferences(
        conn: &mut sqlx::PgConnection,
        tenant_id: Uuid,
        user_id: Uuid,
        req: &UpdatePreferencesRequest,
    ) -> Result<UserPreference, sqlx::Error> {
        sqlx::query_as::<_, UserPreference>(
            r#"
            INSERT INTO notification_preferences (tenant_id, user_id, email_enabled, sms_enabled, push_enabled, in_app_enabled)
            VALUES ($1, $2, COALESCE($3, true), COALESCE($4, true), COALESCE($5, true), COALESCE($6, true))
            ON CONFLICT (user_id) DO UPDATE SET
                tenant_id = EXCLUDED.tenant_id,
                email_enabled = COALESCE($3, notification_preferences.email_enabled),
                sms_enabled = COALESCE($4, notification_preferences.sms_enabled),
                push_enabled = COALESCE($5, notification_preferences.push_enabled),
                in_app_enabled = COALESCE($6, notification_preferences.in_app_enabled),
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(tenant_id)
        .bind(user_id)
        .bind(req.email_enabled)
        .bind(req.sms_enabled)
        .bind(req.push_enabled)
        .bind(req.in_app_enabled)
        .fetch_one(&mut *conn)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NotificationChannel, NotificationPriority};
    use uuid::Uuid;

    #[sqlx::test]
    #[ignore]
    async fn test_preference_filtering_prevents_disabled_channel(pool: sqlx::PgPool) {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // 1. Opt out of Email
        let update_req = UpdatePreferencesRequest {
            email_enabled: Some(false),
            sms_enabled: Some(true),
            push_enabled: Some(true),
            in_app_enabled: Some(true),
        };
        let mut conn = pool.acquire().await.unwrap();
        let _ = NotificationRepo::update_preferences(&mut *conn, tenant_id, user_id, &update_req).await;

        // 2. Try to create Email notification
        let req = CreateNotificationRequest {
            user_id: Some(user_id),
            supplier_id: None,
            order_id: None,
            event_type: None,
            channel: NotificationChannel::Email,
            priority: Some(NotificationPriority::Normal),
            recipient: None,
            subject: None,
            body: "Test".to_string(),
            payload: None,
        };

        let result = NotificationRepo::create(&mut *conn, tenant_id, &req).await;

        // 3. Verify it was rejected
        assert!(result.is_err());
        if let Err(sqlx::Error::Protocol(msg)) = result {
            assert!(msg.to_string().contains("User opted out"));
        } else {
            panic!("Expected Protocol error due to preference filtering");
        }
    }
}
