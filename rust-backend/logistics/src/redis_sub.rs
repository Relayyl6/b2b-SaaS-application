use actix_web::web::Data;
use chrono::Utc;
use platform::{metrics, streams};
use std::env;

use crate::db::LogisticsRepo;
use crate::models::{CreateShipmentRequest, IncomingOrderEvent, LogisticsEvent, ShipmentStatus};
use crate::publisher::RedisPublisher;
use crate::rabbit_pub::RabbitPublisher;

const EVENTS: &[&str] = &["inventory.finalized", "order.cancelled", "logistics.shipment_preparation_command"];

/// Consumes Redis Stream events and applies logistics side effects.
pub async fn listen_to_redis_events(
    repo: Data<LogisticsRepo>,
    redis_pub: Data<RedisPublisher>,
    rabbit_pub: Data<RabbitPublisher>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set in environment")?;
    let consumer = env::var("CONSUMER_NAME").unwrap_or_else(|_| "logistics-1".to_string());

    streams::consume_json::<IncomingOrderEvent, _, _>(
        &redis_url,
        "logistics",
        &consumer,
        EVENTS,
        move |envelope| {
            let repo = repo.clone();
            let redis_pub = redis_pub.clone();
            let rabbit_pub = rabbit_pub.clone();
            async move {
                let event_type = envelope.event_type.clone();
                let event = envelope.payload;

                let tenant_id = envelope.tenant_id.or(event.tenant_id);
                if tenant_id.is_none() || tenant_id == Some(uuid::Uuid::nil()) {
                    tracing::warn!(%event_type, stream = %envelope.stream, "Missing tenant_id in stream event — skipping business logic");
                    metrics::inc_event("logistics", &envelope.stream, &event_type, "tenant_mismatch");
                    return Ok(());
                }

                if let (Some(env_tid), Some(pay_tid)) = (envelope.tenant_id, event.tenant_id) {
                    if env_tid != pay_tid {
                        tracing::warn!(%event_type, ?env_tid, ?pay_tid, "Tenant ID mismatch between envelope and payload — skipping business logic");
                        metrics::inc_event("logistics", &envelope.stream, &event_type, "tenant_mismatch");
                        return Ok(());
                    }
                }

                let result = handle_event(
                    &repo,
                    &redis_pub,
                    &rabbit_pub,
                    &event_type,
                    event,
                )
                .await;
                metrics::inc_event(
                    "logistics",
                    &envelope.stream,
                    &event_type,
                    if result.is_ok() { "ok" } else { "error" },
                );
                if let Err(e) = result {
                    eprintln!("logistics stream handler failed for {event_type}: {e}");
                    return Err(e);
                }
                Ok(())
            }
        },
    )
    .await
}

async fn handle_event(
    repo: &Data<LogisticsRepo>,
    redis_pub: &Data<RedisPublisher>,
    rabbit_pub: &Data<RabbitPublisher>,
    event_type: &str,
    event: IncomingOrderEvent,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event_type {
        "inventory.finalized" | "logistics.shipment_preparation_command" => {
            let Some(order_id) = event.order_id else {
                return Ok(());
            };
            let Some(user_id) = event.user_id else {
                return Ok(());
            };

            match repo.get_by_order_id(order_id).await {
                Ok(_) => return Ok(()),
                Err(sqlx::Error::RowNotFound) => {}
                Err(e) => return Err(Box::new(e)),
            }

            let req = CreateShipmentRequest {
                order_id,
                user_id,
                supplier_id: event.supplier_id,
                product_id: event.product_id,
                notes: Some("Created after payment finalization".to_string()),
            };
            let shipment = repo.create_shipment(&req).await?;
            let outbound = LogisticsEvent {
                tenant_id: event.tenant_id.or(Some(shipment.supplier_id)),
                event_type: "logistics.shipment_created".to_string(),
                shipment_id: shipment.id,
                order_id: shipment.order_id,
                user_id: shipment.user_id,
                supplier_id: shipment.supplier_id,
                product_id: shipment.product_id,
                status: shipment.status,
                tracking_number: shipment.tracking_number,
                timestamp: Utc::now(),
            };
            redis_pub
                .publish("logistics.shipment_created", &outbound)
                .await?;
            rabbit_pub.publish_async(outbound);
        }
        "order.cancelled" => {
            let Some(order_id) = event.order_id else {
                return Ok(());
            };
            let shipment = match repo.cancel_by_order_id(order_id).await {
                Ok(shipment) => shipment,
                Err(sqlx::Error::RowNotFound) => return Ok(()),
                Err(e) => return Err(Box::new(e)),
            };
            let outbound = LogisticsEvent {
                tenant_id: event.tenant_id.or(Some(shipment.supplier_id)),
                event_type: "logistics.shipment_cancelled".to_string(),
                shipment_id: shipment.id,
                order_id: shipment.order_id,
                user_id: shipment.user_id,
                supplier_id: shipment.supplier_id,
                product_id: shipment.product_id,
                status: ShipmentStatus::Cancelled,
                tracking_number: shipment.tracking_number,
                timestamp: Utc::now(),
            };
            redis_pub.publish_async("logistics.shipment_cancelled", outbound.clone());
            rabbit_pub.publish_async(outbound);
        }
        _ => {}
    }

    Ok(())
}
