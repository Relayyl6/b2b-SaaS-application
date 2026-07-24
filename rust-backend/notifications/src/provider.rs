use crate::models::{Notification, NotificationChannel};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::env;

#[derive(Clone)]
pub struct NotificationProvider {
    dry_run: bool,
    client: reqwest::Client,
    email_webhook_url: Option<String>,
    sms_webhook_url: Option<String>,
    expo_push_url: String,
    expo_access_token: Option<String>,
}

impl NotificationProvider {
    pub fn from_env() -> Self {
        Self {
            dry_run: env::var("NOTIFICATION_DRY_RUN")
                .map(|value| value != "false")
                .unwrap_or(true),
            client: reqwest::Client::new(),
            email_webhook_url: env_non_empty("EMAIL_WEBHOOK_URL"),
            sms_webhook_url: env_non_empty("SMS_WEBHOOK_URL"),
            expo_push_url: env::var("EXPO_PUSH_URL")
                .unwrap_or_else(|_| "https://exp.host/--/api/v2/push/send".to_string()),
            expo_access_token: env_non_empty("EXPO_ACCESS_TOKEN"),
        }
    }

    pub async fn send(&self, notification: &Notification) -> Result<(), String> {
        if self.dry_run {
            println!(
                "notification dry-run: {:?} to {} subject={:?}",
                notification.channel, notification.recipient, notification.subject
            );
            return Ok(());
        }

        match &notification.channel {
            NotificationChannel::InApp => Ok(()),
            NotificationChannel::Email => self.send_email(notification).await,
            NotificationChannel::Sms => self.send_sms(notification).await,
            NotificationChannel::Push => self.send_push(notification).await,
        }
    }

    async fn send_email(&self, notification: &Notification) -> Result<(), String> {
        let Some(url) = &self.email_webhook_url else {
            return Err("EMAIL_WEBHOOK_URL is not configured".to_string());
        };

        let response = self
            .client
            .post(url)
            .json(&json!({
                "to": notification.recipient,
                "subject": notification.subject,
                "text": notification.body,
                "html": notification.payload.get("html"),
                "metadata": &notification.payload,
            }))
            .send()
            .await
            .map_err(|e| format!("email webhook request failed: {e}"))?;

        ensure_success(response, "email webhook").await
    }

    async fn send_sms(&self, notification: &Notification) -> Result<(), String> {
        let Some(url) = &self.sms_webhook_url else {
            return Err("SMS_WEBHOOK_URL is not configured".to_string());
        };

        let response = self
            .client
            .post(url)
            .json(&json!({
                "to": notification.recipient,
                "message": notification.body,
                "metadata": &notification.payload,
            }))
            .send()
            .await
            .map_err(|e| format!("sms webhook request failed: {e}"))?;

        ensure_success(response, "sms webhook").await
    }

    async fn send_push(&self, notification: &Notification) -> Result<(), String> {
        let push_token = notification
            .payload
            .get("push_token")
            .and_then(|value| value.as_str())
            .unwrap_or(notification.recipient.as_str());

        if push_token.trim().is_empty() || push_token.starts_with("user:") {
            return Err("push notification requires a registered device token".to_string());
        }

        let mut request = self
            .client
            .post(&self.expo_push_url)
            .header(CONTENT_TYPE, "application/json")
            .json(&json!({
                "to": push_token,
                "title": notification.subject.as_deref().unwrap_or("Notification"),
                "body": notification.body,
                "sound": notification.payload.get("sound").and_then(|v| v.as_str()).unwrap_or("default"),
                "priority": notification.priority,
                "data": {
                    "notification_id": notification.id,
                    "event_type": notification.event_type,
                    "order_id": notification.order_id,
                    "supplier_id": notification.supplier_id,
                    "payload": &notification.payload
                }
            }));

        if let Some(token) = &self.expo_access_token {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("expo push request failed: {e}"))?;

        ensure_success(response, "expo push").await
    }
}

async fn ensure_success(response: reqwest::Response, label: &str) -> Result<(), String> {
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }

    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "<body unavailable>".to_string());
    Err(format!("{label} failed with {status}: {body}"))
}

fn env_non_empty(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.trim().is_empty())
}
