use actix_web::web;
use platform::{metrics, streams};
use std::env;

use crate::db::NotificationRepo;
use crate::models::{
    CreateNotificationRequest, DomainEvent, NotificationChannel, NotificationPriority, UserPreference
};

const EVENTS: &[&str] = &[
    "order.created",
    "order.cancelled",
    "order.confirmed",
    "order.failed",
    "order.delivered",
    "order.review_requested",
    "inventory.lowstock",
    "inventory.rejected",
    "logistics.shipment_created",
    "logistics.shipment_updated",
    "logistics.shipment_cancelled",
    "payment.failed",
    "payment.success",
    "payment.cancelled",
    "supplier.created",
    "supplier.status_updated",
    "user.created",
];

pub async fn listen_to_redis_events(
    repo: web::Data<NotificationRepo>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL")?;
    let consumer = env::var("CONSUMER_NAME").unwrap_or_else(|_| "notifications-1".to_string());

    streams::consume_json::<DomainEvent, _, _>(
        &redis_url,
        "notifications",
        &consumer,
        EVENTS,
        move |envelope| {
            let repo = repo.clone();
            async move {
                let event_type = envelope.event_type.clone();
                let event = &envelope.payload;
                let tenant_id = envelope.tenant_id.or(event.tenant_id);

                if tenant_id.is_none() || tenant_id == Some(uuid::Uuid::nil()) {
                    tracing::warn!(%event_type, stream = %envelope.stream, "Missing tenant_id in notification stream event — skipping notification");
                    metrics::inc_event("notifications", &envelope.stream, &event_type, "tenant_mismatch");
                    return Ok(());
                }

                if let (Some(env_tid), Some(pay_tid)) = (envelope.tenant_id, event.tenant_id) {
                    if env_tid != pay_tid {
                        tracing::warn!(%event_type, ?env_tid, ?pay_tid, "Tenant ID mismatch between envelope and payload — skipping notification");
                        metrics::inc_event("notifications", &envelope.stream, &event_type, "tenant_mismatch");
                        return Ok(());
                    }
                }

                let Some((subject, body, priority)) =
                    notification_from_event(&envelope.event_type, &envelope.payload)
                else {
                    return Ok(());
                };
                
                let event = &envelope.payload;
                let recipient = event
                    .recipient
                    .clone()
                    .or_else(|| event.user_id.map(|id| format!("user:{id}")))
                    .or_else(|| event.supplier_id.map(|id| format!("supplier:{id}")))
                    .unwrap_or_else(|| "system".to_string());

                // Fetch user preferences dynamically
                let prefs = if let Some(uid) = event.user_id {
                    repo.get_preferences(uid).await.unwrap_or_else(|_| UserPreference {
                        user_id: uid,
                        tenant_id: event.tenant_id.unwrap_or_else(uuid::Uuid::nil),
                        email_enabled: true,
                        sms_enabled: false,
                        push_enabled: false,
                        in_app_enabled: true,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    })
                } else {
                    UserPreference {
                        user_id: uuid::Uuid::new_v4(),
                        tenant_id: event.tenant_id.unwrap_or_else(uuid::Uuid::nil),
                        email_enabled: true,
                        sms_enabled: false,
                        push_enabled: false,
                        in_app_enabled: true,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }
                };

                // Fan out to all actively enabled channels!
                let mut active_channels = Vec::new();
                if prefs.email_enabled { active_channels.push(NotificationChannel::Email); }
                if prefs.sms_enabled { active_channels.push(NotificationChannel::Sms); }
                if prefs.push_enabled { active_channels.push(NotificationChannel::Push); }
                if prefs.in_app_enabled { active_channels.push(NotificationChannel::InApp); }
                
                if active_channels.is_empty() {
                    active_channels.push(NotificationChannel::InApp);
                }

                for channel in active_channels {
                    let req = CreateNotificationRequest {
                        user_id: event.user_id,
                        supplier_id: event.supplier_id,
                        order_id: event.order_id,
                        event_type: Some(envelope.event_type.clone()),
                        channel,
                        priority: Some(priority.clone()),
                        recipient: Some(recipient.clone()),
                        subject: subject.clone(),
                        body: body.clone(),
                        payload: Some(event.payload.clone()),
                    };

                    match repo.create(&req).await {
                        Ok(_) => {
                            metrics::inc_event("notifications", &envelope.stream, &envelope.event_type, "ok");
                        }
                        Err(e) => {
                            eprintln!("Failed to create notification for channel: {}", e);
                            metrics::inc_event("notifications", &envelope.stream, &envelope.event_type, "error");
                        }
                    }
                }

                Ok(())
            }
        },
    )
    .await
}

fn notification_from_event(channel: &str, event: &DomainEvent) -> Option<(Option<String>, String, NotificationPriority)> {
    match channel {
        "inventory.lowstock" => Some((
            Some("Inventory is running low".to_string()),
            format!("Product {:?} is at or below its low-stock threshold.", event.product_id),
            NotificationPriority::High,
        )),
        "inventory.rejected" => Some((
            Some("Order could not be reserved".to_string()),
            format!("Order {:?} was rejected because stock was unavailable.", event.order_id),
            NotificationPriority::High,
        )),
        "logistics.shipment_created" => Some((
            Some("Shipment created".to_string()),
            format!("Shipment for order {:?} was created. Tracking number: {}", event.order_id, event.tracking_number.as_deref().unwrap_or("pending")),
            NotificationPriority::Normal,
        )),
        "logistics.shipment_updated" => Some((
            Some("Shipment updated".to_string()),
            format!("Shipment for order {:?} is now {}.", event.order_id, event.status.as_deref().unwrap_or("updated")),
            NotificationPriority::Normal,
        )),
        "logistics.shipment_cancelled" | "order.cancelled" => Some((
            Some("Order cancelled".to_string()),
            format!("Order {:?} was cancelled.", event.order_id),
            NotificationPriority::Normal,
        )),
        "payment.failed" => Some((
            Some("Payment failed".to_string()),
            format!("Payment for order {:?} failed.", event.order_id),
            NotificationPriority::Critical,
        )),
        "payment.success" => Some((
            Some("Payment successful".to_string()),
            format!("Payment for order {:?} was successful.", event.order_id),
            NotificationPriority::Normal,
        )),
        "payment.cancelled" => Some((
            Some("Payment cancelled".to_string()),
            format!("Payment for order {:?} was cancelled.", event.order_id),
            NotificationPriority::High,
        )),
        "supplier.created" => Some((
            Some("Supplier onboarding started".to_string()),
            format!("Supplier {:?} has been created and is pending review.", event.supplier_id),
            NotificationPriority::Normal,
        )),
        "supplier.status_updated" => Some((
            Some("Supplier status updated".to_string()),
            format!("Supplier {:?} status is now {}.", event.supplier_id, event.status.as_deref().unwrap_or("updated")),
            NotificationPriority::Normal,
        )),
        "user.created" => Some((
            Some("Welcome".to_string()),
            "Your account has been created successfully.".to_string(),
            NotificationPriority::Normal,
        )),
        "order.created" => Some((
            Some("Order received".to_string()),
            format!("Order {:?} has been received and is pending inventory reservation.", event.order_id),
            NotificationPriority::Normal,
        )),
        "order.confirmed" => Some((
            Some("Order Confirmed".to_string()),
            format!("Your order {:?} has been confirmed and is being prepared for shipment.", event.order_id),
            NotificationPriority::High,
        )),
        "order.failed" => Some((
            Some("Order Failed".to_string()),
            format!("We're sorry, but your order {:?} has failed.", event.order_id),
            NotificationPriority::High,
        )),
        "order.delivered" => Some((
            Some("Order Delivered".to_string()),
            format!("Your order {:?} has been delivered.", event.order_id),
            NotificationPriority::Normal,
        )),
        "order.review_requested" => Some((
            Some("Please review your purchase".to_string()),
            format!("How did you like your order {:?}? Please leave a review.", event.order_id),
            NotificationPriority::Normal,
        )),
        _ => None,
    }
}
