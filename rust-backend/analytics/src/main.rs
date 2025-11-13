use redis::{AsyncCommands, RedisResult, from_redis_value};
use redis::streams::{StreamReadOptions, StreamReadReply};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use reqwest::Client;
use std::collections::HashMap;
use std::env;
use tokio::task;

#[derive(Debug, Serialize, Deserialize)]
struct AnalyticsEvent {
    event_type: String,
    source_stream: String,
    payload: Value,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProductEvent {
    event_type: String, // "Product.created", "Product.updated", etc.
    product_id: Option<String>,
    supplier_id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    price: Option<f64>,
    category: Option<String>,
    quantity: Option<i32>,
    low_stock_threshold: Option<i32>,
    unit: Option<String>,
    quantity_change: Option<i32>,
    available: Option<bool>,
}

async fn index_event_to_opensearch(
    event: &AnalyticsEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    let opensearch_url = env::var("OPENSEARCH_URL").expect("OPENSEARCH_URL not set");
    let index_name = env::var("OPENSEARCH_INDEX").unwrap_or("platform_analytics".to_string());

    let client = Client::new();
    let url = format!("{}/{}/_doc", opensearch_url, index_name);

    let res = client.post(&url).json(event).send().await?;
    if !res.status().is_success() {
        eprintln!("⚠️ Failed to index event: {:?}", res.text().await?);
    } else {
        println!("✅ Indexed event: {}", event.event_type);
    }

    Ok(())
}

async fn consume_stream(stream: &str, allowed_events: Vec<&str>) {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set");
    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let mut con = client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    let mut last_id = "0".to_string();
    println!("📡 Listening to stream: {}", stream);

    loop {
        let opts = StreamReadOptions::default().block(0);
        let reply: RedisResult<StreamReadReply> = redis::cmd("XREAD")
            .arg("BLOCK")
            .arg(0)
            .arg("STREAMS")
            .arg(stream)
            .arg(&last_id)
            .query_async(&mut con)
            .await;

        let reply = match reply {
            Ok(r) => r,
            Err(e) => {
                eprintln!("❌ Redis read error on {}: {}", stream, e);
                continue;
            }
        };

        for key in reply.keys {
            for msg in key.ids {
                last_id = msg.id.clone();
                let mut payload_map = serde_json::Map::new();

                for (field, value) in msg.map {
                    let val_str: String = from_redis_value(&value).unwrap_or_else(|_| format!("{:?}", value));
                    payload_map.insert(field.clone(), json!(val_str));
                }

                // Try to extract event type (if available in payload)
                let event_type = payload_map
                    .get("event_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Only process events that belong to this stream
                if !allowed_events.contains(&event_type.as_str()) {
                    println!("🚫 Ignored unrelated event: {}", event_type);
                    continue;
                }

                let analytics_event = AnalyticsEvent {
                    event_type: event_type.clone(),
                    source_stream: stream.to_string(),
                    payload: Value::Object(payload_map.clone()),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                if let Err(e) = index_event_to_opensearch(&analytics_event).await {
                    eprintln!("❌ Failed to index event from {}: {}", stream, e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Define event groups (service → events)
    let event_groups: HashMap<&str, Vec<&str>> = HashMap::from([
        (
            "product_stream",
            vec![
                "Product.created",
                "Product.updated",
                "Product.deleted",
            ],
        ),
        (
            "user_stream",
            vec![
                "User.created",
                "User.deleted",
                "User.sign_up",
                "User.sign_in",
                "User.sign_out",
                "User.updated",
            ],
        ),
        (
            "orders_stream",
            vec![
                "Order.created",
                "Order.cancelled",
                "Order.completed",
                "Product.sold",
                "Product.bought"
            ],
        ),
        (
            "payments_stream",
            vec![
                "Payments.processed",
                "Payments.failed"
            ],
        ),
        (
            "inventory_stream",
            vec![
                "Inventory.low_stock", 
                "Inventory.updated"   
            ],
        ),
        (
            "suppliers_stream",
            vec![
                "Supplier.created",
                "Supplier.deleted"
            ],
        ),
    ]);

    // Spawn a consumer task for each stream
    for (stream, events) in event_groups {
        let s = stream.to_string();
        let e = events.clone();
        task::spawn(async move {
            consume_stream(&s, e).await;
        });
    }

    // Keep running forever
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
