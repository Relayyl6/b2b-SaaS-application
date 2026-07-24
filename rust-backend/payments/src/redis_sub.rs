use actix_web::web;
use platform::{metrics, streams};
use std::env;
use uuid::Uuid;

use crate::db::PaymentRepo;
use crate::models::CreatePaymentIntentRequest;

// We need a subset of the ProductEvent/OrderEvent to parse the payload
#[derive(Debug, serde::Deserialize)]
pub struct OrderContextEvent {
    pub order_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub quantity: Option<i32>,
    pub price: Option<f64>,
}

const EVENTS: &[&str] = &["inventory.reserved", "order.cancelled"];

pub async fn listen_to_redis_events(
    repo: web::Data<PaymentRepo>,
) -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set")?;
    let consumer = env::var("CONSUMER_NAME").unwrap_or_else(|_| "payments-1".to_string());

    streams::consume_json::<OrderContextEvent, _, _>(
        &redis_url,
        "payments",
        &consumer,
        EVENTS,
        move |envelope| {
            let repo = repo.clone();
            async move {
                let event_type = envelope.event_type.clone();
                let result = handle_event(&repo, &event_type, envelope.payload).await;

                metrics::inc_event(
                    "payments",
                    &envelope.stream,
                    &event_type,
                    if result.is_ok() { "ok" } else { "error" },
                );

                if let Err(e) = result {
                    eprintln!("Payments stream handler failed for {event_type}: {e}");
                }
            }
        },
    )
    .await
}

async fn handle_event(
    repo: &PaymentRepo,
    event_type: &str,
    event: OrderContextEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    match event_type {
        "inventory.reserved" => {
            let order_id = event.order_id.ok_or("Missing order_id")?;
            let user_id = event.user_id.unwrap_or_default();
            let supplier_id = event.supplier_id.unwrap_or_default();
            let product_id = event.product_id.unwrap_or_default();
            let quantity = event.quantity.unwrap_or(1);
            let price = event.price.unwrap_or(100.0);
            let amount = (quantity as f64) * price;

            let req = CreatePaymentIntentRequest {
                idempotency_key: format!("auto_intent_{}", order_id),
                order_id,
                user_id,
                supplier_id,
                product_id,
                quantity,
                amount,
                currency: Some("USD".to_string()),
                provider: Some("system".to_string()),
                metadata: None,
            };

            repo.create_intent(&req).await?;
            println!("Auto-generated PaymentIntent for order {}", order_id);
        }
        "order.cancelled" => {
            let order_id = event.order_id.ok_or("Missing order_id")?;
            
            // Note: Currently PaymentRepo doesn't have cancel_by_order_id,
            // so we execute a raw query to update the intent status.
            // Let's add a method to PaymentRepo, or use raw sqlx if pool is accessible.
            // For now, we will add cancel_by_order_id to PaymentRepo in db.rs
            repo.cancel_by_order_id(order_id).await?;
            println!("Cancelled PaymentIntent for order {}", order_id);
        }
        _ => {}
    }

    Ok(())
}
