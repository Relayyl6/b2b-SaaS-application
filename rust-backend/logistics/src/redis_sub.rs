use actix_web::web::Data;
use chrono::Utc;
use futures_util::StreamExt;
use redis::{aio::Connection, Client};
use std::env;

use crate::db::LogisticsRepo;
use crate::models::{CreateShipmentRequest, IncomingOrderEvent, LogisticsEvent, ShipmentStatus};
use crate::publisher::RedisPublisher;
use crate::rabbit_pub::RabbitPublisher;

/// Listens to Redis pub/sub channels and applies logistics side effects by creating or cancelling shipments.
///
/// This function continuously connects to Redis (configured via the `REDIS_URL` environment variable),
/// subscribes to the `inventory.reserved` and `order.cancelled` channels, processes incoming messages as
/// `IncomingOrderEvent`, performs repository operations via the provided `LogisticsRepo`, and publishes outbound
/// `LogisticsEvent`s via the provided `RedisPublisher`.
///
/// Behaviour summary:
/// - Requires `REDIS_URL` to be set in the environment; returns an error if it is missing.
/// - On `inventory.reserved`: if `order_id` and `user_id` are present and no shipment exists for the order,
///   creates a shipment and publishes a `logistics.shipment_created` event.
/// - On `order.cancelled`: if `order_id` is present, cancels the shipment by order ID and publishes a
///   `logistics.shipment_cancelled` event. A "row not found" cancellation is treated as a no-op.
///
/// # Examples
///
/// ```no_run
/// use actix_web::web::Data;
///
/// // Provided `repo` and `redis_pub` should be initialized application singletons.
/// // tokio::spawn can be used to run the listener in the background.
/// # async fn example(repo: Data<crate::LogisticsRepo>, redis_pub: Data<crate::RedisPublisher>) {
/// tokio::spawn(async move {
///     let _ = crate::listen_to_redis_events(repo, redis_pub).await;
/// });
/// # }
/// ```
// TODO(redis): remove once redis async pubsub API replacement is adopted across services (target Q2 2026).
// Using deprecated `Client::get_async_connection` for compatibility with current redis crate usage.
#[allow(deprecated)]
/// Consumes Redis pub/sub events and applies logistics side effects.
pub async fn listen_to_redis_events(
    repo: Data<LogisticsRepo>,
    redis_pub: Data<RedisPublisher>,
    rabbit_pub: Data<RabbitPublisher>,
) -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = env::var("REDIS_URL").map_err(|_| "REDIS_URL must be set in environment")?;

    loop {
        let client = Client::open(redis_url.as_str())?;
        let conn: Connection = match client.get_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to connect redis: {e:?}");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        let mut pubsub = conn.into_pubsub();
        let mut subscribed = true;
        for channel in ["inventory.reserved", "order.cancelled"] {
            if let Err(e) = pubsub.subscribe(channel).await {
                eprintln!("Failed to subscribe to {channel}: {e:?}");
                subscribed = false;
                break;
            }
        }
        if !subscribed {
            drop(pubsub);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            continue;
        }

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            let channel = msg.get_channel_name().to_string();
            let payload: String = match msg.get_payload() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let event: IncomingOrderEvent = match serde_json::from_str(&payload) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Invalid incoming logistics event payload: {e}");
                    continue;
                }
            };

            let route_key = if channel.is_empty() {
                event.event_type.as_str()
            } else {
                channel.as_str()
            };

            match route_key {
                "inventory.reserved" => {
                    let Some(order_id) = event.order_id else {
                        eprintln!(
                            "Skipping inventory.reserved event: missing order_id (event_type={})",
                            event.event_type
                        );
                        continue;
                    };
                    let Some(user_id) = event.user_id else {
                        eprintln!("Skipping inventory.reserved event: missing user_id for order {order_id}");
                        continue;
                    };
                    if event.supplier_id.is_nil() {
                        eprintln!("Skipping inventory.reserved event: nil supplier_id for order {order_id}");
                        continue;
                    }

                    match repo.get_by_order_id(order_id).await {
                        Ok(_) => continue,
                        Err(sqlx::Error::RowNotFound) => {}
                        Err(e) => {
                            eprintln!(
                                "Failed checking shipment existence for order {order_id}: {e:?}"
                            );
                            continue;
                        }
                    }

                    let req = CreateShipmentRequest {
                        order_id,
                        user_id,
                        supplier_id: event.supplier_id,
                        product_id: event.product_id,
                        notes: Some("Created from inventory reservation".to_string()),
                    };

                    match repo.create_shipment(&req).await {
                        Ok(shipment) => {
                            let outbound = LogisticsEvent {
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

                            if let Err(e) = redis_pub
                                .publish("logistics.shipment_created", &outbound)
                                .await
                            {
                                eprintln!(
                                    "Failed publishing logistics.shipment_created event: {e:?}"
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed creating shipment for order {order_id}: {e:?}");
                        }
                    }
                }
                "order.cancelled" => {
                    if let Some(order_id) = event.order_id {
                        match repo.cancel_by_order_id(order_id).await {
                            Ok(shipment) => {
                                let outbound = LogisticsEvent {
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

                                redis_pub.publish_async(
                                    "logistics.shipment_cancelled",
                                    outbound.clone(),
                                );
                                rabbit_pub.publish_async(outbound.clone());
                            }
                            Err(sqlx::Error::RowNotFound) => {}
                            Err(e) => eprintln!("Failed to cancel shipment by order id: {e:?}"),
                        }
                    }
                }
                _ => {}
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
