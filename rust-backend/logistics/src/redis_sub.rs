use actix_web::web::Data;
use chrono::Utc;
use futures_util::StreamExt;
use redis::{aio::Connection, Client};
use std::env;

use crate::db::LogisticsRepo;
use crate::models::{CreateShipmentRequest, IncomingOrderEvent, LogisticsEvent, ShipmentStatus};
use crate::publisher::RedisPublisher;

#[allow(deprecated)]
/// Consumes Redis pub/sub events and applies logistics side effects.
pub async fn listen_to_redis_events(
    repo: Data<LogisticsRepo>,
    redis_pub: Data<RedisPublisher>,
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
        for channel in ["inventory.reserved", "order.cancelled"] {
            if let Err(e) = pubsub.subscribe(channel).await {
                eprintln!("Failed to subscribe to {channel}: {e:?}");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
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
                        continue;
                    };
                    let Some(user_id) = event.user_id else {
                        continue;
                    };

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

                    if let Ok(shipment) = repo.create_shipment(&req).await {
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
                            eprintln!("Failed publishing logistics.shipment_created event: {e:?}");
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

                                if let Err(e) = redis_pub
                                    .publish("logistics.shipment_cancelled", &outbound)
                                    .await
                                {
                                    eprintln!("Failed publishing logistics.shipment_cancelled event: {e:?}");
                                }
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
