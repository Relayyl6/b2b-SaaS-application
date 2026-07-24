use actix_web::web;
use platform::{metrics, streams};
use std::env;

use crate::db::NotificationRepo;
use crate::models::{
    CreateNotificationRequest, DomainEvent, NotificationChannel, NotificationPriority,
};

const EVENTS: &[&str] = &[
    "order.created",
    "order.cancelled",
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
) -> Result<(), Box<dyn std::error::Error>> {
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
                let Some(mut notification) =
                    notification_from_event(&envelope.event_type, envelope.payload)
                else {
                    return;
                };
                notification.event_type = Some(envelope.event_type.clone());
                let outcome = if repo.create(&notification).await.is_ok() {
                    "ok"
                } else {
                    "error"
                };
                metrics::inc_event(
                    "notifications",
                    &envelope.stream,
                    &envelope.event_type,
                    outcome,
                );
            }
        },
    )
    .await
}

fn notification_from_event(channel: &str, event: DomainEvent) -> Option<CreateNotificationRequest> {
    let recipient = event
        .recipient
        .clone()
        .or_else(|| event.user_id.map(|id| format!("user:{id}")))
        .or_else(|| event.supplier_id.map(|id| format!("supplier:{id}")))?;

    let (subject, body, priority) = match channel {
        "inventory.lowstock" => (
            Some("Inventory is running low".to_string()),
            format!(
                "Product {:?} is at or below its low-stock threshold.",
                event.product_id
            ),
            NotificationPriority::High,
        ),
        "inventory.rejected" => (
            Some("Order could not be reserved".to_string()),
            format!(
                "Order {:?} was rejected because stock was unavailable.",
                event.order_id
            ),
            NotificationPriority::High,
        ),
        "logistics.shipment_created" => (
            Some("Shipment created".to_string()),
            format!(
                "Shipment for order {:?} was created. Tracking number: {}",
                event.order_id,
                event
                    .tracking_number
                    .unwrap_or_else(|| "pending".to_string())
            ),
            NotificationPriority::Normal,
        ),
        "logistics.shipment_updated" => (
            Some("Shipment updated".to_string()),
            format!(
                "Shipment for order {:?} is now {}.",
                event.order_id,
                event.status.unwrap_or_else(|| "updated".to_string())
            ),
            NotificationPriority::Normal,
        ),
        "logistics.shipment_cancelled" | "order.cancelled" => (
            Some("Order cancelled".to_string()),
            format!("Order {:?} was cancelled.", event.order_id),
            NotificationPriority::Normal,
        ),
        "payment.failed" => (
            Some("Payment failed".to_string()),
            format!("Payment for order {:?} failed.", event.order_id),
            NotificationPriority::Critical,
        ),
        "payment.success" => (
            Some("Payment successful".to_string()),
            format!("Payment for order {:?} was successful.", event.order_id),
            NotificationPriority::Normal,
        ),
        "payment.cancelled" => (
            Some("Payment cancelled".to_string()),
            format!("Payment for order {:?} was cancelled.", event.order_id),
            NotificationPriority::High,
        ),
        "supplier.created" => (
            Some("Supplier onboarding started".to_string()),
            format!(
                "Supplier {:?} has been created and is pending review.",
                event.supplier_id
            ),
            NotificationPriority::Normal,
        ),
        "supplier.status_updated" => (
            Some("Supplier status updated".to_string()),
            format!(
                "Supplier {:?} status is now {}.",
                event.supplier_id,
                event.status.unwrap_or_else(|| "updated".to_string())
            ),
            NotificationPriority::Normal,
        ),
        "user.created" => (
            Some("Welcome".to_string()),
            "Your account has been created successfully.".to_string(),
            NotificationPriority::Normal,
        ),
        "order.created" => (
            Some("Order received".to_string()),
            format!(
                "Order {:?} has been received and is pending inventory reservation.",
                event.order_id
            ),
            NotificationPriority::Normal,
        ),
        _ => return None,
    };

    Some(CreateNotificationRequest {
        user_id: event.user_id,
        supplier_id: event.supplier_id,
        order_id: event.order_id,
        event_type: Some(event.event_type.unwrap_or_else(|| channel.to_string())),
        channel: NotificationChannel::InApp,
        priority: Some(priority),
        recipient: Some(recipient),
        subject,
        body,
        payload: Some(event.payload),
    })
}
