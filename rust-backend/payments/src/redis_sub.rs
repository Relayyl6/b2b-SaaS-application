use actix_web::web;
use platform::{metrics, streams};
use std::env;
use uuid::Uuid;

use crate::db::PaymentRepo;
use crate::models::CreatePaymentIntentRequest;
use crate::stripe::StripeClient;

// We need a subset of the ProductEvent/OrderEvent to parse the payload
#[derive(Debug, serde::Deserialize)]
pub struct OrderContextEvent {
    pub tenant_id: Option<Uuid>,
    pub order_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub quantity: Option<i32>,
    pub price: Option<f64>,
}

const EVENTS: &[&str] = &["inventory.reserved", "order.cancelled", "order.refunded", "order.delivered", "payment.refund_command"];

pub async fn listen_to_redis_events(
    repo: web::Data<PaymentRepo>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set")?;
    let consumer = env::var("CONSUMER_NAME").unwrap_or_else(|_| "payments-1".to_string());
    let stripe_client = StripeClient::new();

    streams::consume_json::<OrderContextEvent, _, _>(
        &redis_url,
        "payments",
        &consumer,
        EVENTS,
        move |envelope| {
            let repo = repo.clone();
            let stripe_client = stripe_client.clone();
            async move {
                let event_type = envelope.event_type.clone();
                let event = &envelope.payload;
                let tenant_id = envelope.tenant_id.or(event.tenant_id);

                if tenant_id.is_none() || tenant_id == Some(uuid::Uuid::nil()) {
                    tracing::warn!(%event_type, stream = %envelope.stream, "Missing tenant_id in payment stream event — skipping business logic");
                    metrics::inc_event("payments", &envelope.stream, &event_type, "tenant_mismatch");
                    return Ok(());
                }

                if let (Some(env_tid), Some(pay_tid)) = (envelope.tenant_id, event.tenant_id) {
                    if env_tid != pay_tid {
                        tracing::warn!(%event_type, ?env_tid, ?pay_tid, "Tenant ID mismatch between envelope and payload — skipping business logic");
                        metrics::inc_event("payments", &envelope.stream, &event_type, "tenant_mismatch");
                        return Ok(());
                    }
                }

                let result = handle_event(&repo, &stripe_client, &event_type, envelope.payload).await;

                metrics::inc_event(
                    "payments",
                    &envelope.stream,
                    &event_type,
                    if result.is_ok() { "ok" } else { "error" },
                );

                if let Err(e) = result {
                    eprintln!("Payments stream handler failed for {event_type}: {e}");
                    return Err(e);
                }
                Ok(())
            }
        },
    )
    .await
}

async fn handle_event(
    repo: &PaymentRepo,
    stripe_client: &StripeClient,
    event_type: &str,
    event: OrderContextEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event_type {
        "inventory.reserved" => {
            let order_id = event.order_id.ok_or("Missing order_id")?;
            let user_id = event.user_id.unwrap_or_default();
            let supplier_id = event.supplier_id.unwrap_or_default();
            let product_id = event.product_id.unwrap_or_default();
            let quantity = event.quantity.unwrap_or(1);
            let price = event.price.unwrap_or(100.0);
            let amount = ((quantity as f64) * price * 100.0) as i64;
            let idempotency_key = format!("auto_intent_{}", order_id);

            let stripe_res = stripe_client
                .create_payment_intent(amount, "usd", None, &idempotency_key)
                .await?;

            let req = CreatePaymentIntentRequest {
                idempotency_key,
                order_id,
                user_id,
                supplier_id,
                product_id,
                quantity,
                amount,
                currency: Some("USD".to_string()),
                provider: Some("stripe".to_string()),
                metadata: Some(serde_json::json!({
                    "client_secret": stripe_res.client_secret,
                    "stripe_id": stripe_res.id,
                })),
            };

            repo.create_intent(&req).await?;
            println!("Auto-generated PaymentIntent for order {}", order_id);
        }
        "order.cancelled" | "payment.refund_command" => {
            let order_id = event.order_id.ok_or("Missing order_id")?;
            if let Ok(intent) = repo.get_intent_by_order_id(order_id).await {
                if intent.status == crate::models::PaymentStatus::Succeeded {
                    // It succeeded already, so we must refund, not cancel
                    if let Some(stripe_id) = intent.provider_reference {
                        if let Err(e) = stripe_client.refund_payment(&stripe_id, None, Some(&intent.id.to_string())).await {
                            eprintln!("Failed to refund intent in Stripe: {e}");
                        } else {
                            repo.update_status(intent.id, crate::models::PaymentStatus::Refunded).await?;
                            println!("Refunded PaymentIntent for order {}", order_id);
                        }
                    }
                } else if intent.status != crate::models::PaymentStatus::Cancelled {
                    // Not succeeded, not cancelled, we can cancel
                    if let Some(stripe_id) = intent.provider_reference {
                        if let Err(e) = stripe_client.cancel_payment_intent(&stripe_id).await {
                            eprintln!("Failed to cancel intent in Stripe: {e}");
                        }
                    }
                    repo.cancel_by_order_id(order_id).await?;
                    println!("Cancelled PaymentIntent for order {}", order_id);
                }
            }
        }
        "order.refunded" => {
            let order_id = event.order_id.ok_or("Missing order_id")?;
            if let Ok(intent) = repo.get_intent_by_order_id(order_id).await {
                if let Some(stripe_id) = intent.provider_reference {
                    if let Err(e) = stripe_client.refund_payment(&stripe_id, None, Some(&intent.id.to_string())).await {
                        eprintln!("Failed to refund intent in Stripe: {e}");
                    } else {
                        // Mark as refunded in DB
                        repo.update_status(intent.id, crate::models::PaymentStatus::Refunded).await?;
                        println!("Refunded PaymentIntent for order {}", order_id);
                    }
                }
            }
        }
        "order.delivered" => {
            let order_id = event.order_id.ok_or("Missing order_id")?;
            if let Ok(intent) = repo.get_intent_by_order_id(order_id).await {
                let amount_cents = intent.amount;
                
                // Deduct 5% platform fee
                let platform_fee = (amount_cents as f64 * 0.05) as i64;
                let payout_amount = amount_cents - platform_fee;

                // Fetch real stripe connect account ID from supplier-management
                let supplier_url = env::var("SUPPLIER_MANAGEMENT_URL")
                    .unwrap_or_else(|_| "http://localhost:3002".to_string());
                
                let res = reqwest::Client::new()
                    .get(&format!("{}/suppliers/{}", supplier_url, intent.supplier_id))
                    .send()
                    .await;

                let stripe_account_id = match res {
                    Ok(r) if r.status().is_success() => {
                        let json: serde_json::Value = r.json().await.unwrap_or_default();
                        json["stripe_connect_id"].as_str().unwrap_or(&format!("acct_mock_{}", intent.supplier_id)).to_string()
                    }
                    _ => format!("acct_mock_{}", intent.supplier_id)
                };

                match stripe_client.transfer_to_supplier(payout_amount, &intent.currency, &stripe_account_id, Some(&intent.id.to_string())).await {
                    Ok(tr_id) => println!("Transferred {} cents to {} (Transfer ID: {})", payout_amount, stripe_account_id, tr_id),
                    Err(e) => eprintln!("Failed to transfer to supplier: {}", e),
                }
            }
        }
        _ => {}
    }

    Ok(())
}
