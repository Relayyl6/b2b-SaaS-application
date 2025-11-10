use redis::{AsyncCommands, RedisResult, from_redis_value};
use redis::streams::{StreamReadOptions, StreamReadReply};
use serde::{Deserialize, Serialize};
use serde_json::json;
use reqwest::Client;
use std::env;
use tokio::task;

#[derive(Debug, Serialize, Deserialize)]
struct AnalyticsEvent {
    event_type: String,
    payload: serde_json::Value,
    timestamp: String,
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

async fn consume_stream(stream: &str) {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set");
    let client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let mut con = client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    let mut last_id = "0".to_string();

    println!("🎧 Listening to stream: {}", stream);

    loop {
        // Use the high-level XREAD API
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
                eprintln!("❌ Redis read error: {}", e);
                continue;
            }
        };

        for key in reply.keys {
            for msg in key.ids {
                last_id = msg.id.clone();

                // Convert message fields into a JSON map safely
                let mut payload = serde_json::Map::new();
                for (field, value) in msg.map {
                    // Convert redis::Value into String using FromRedisValue
                    let val_str: String = match from_redis_value(&value) {
                        Ok(v) => v,
                        Err(_) => format!("{:?}", value), // fallback if not convertible
                    };
                    payload.insert(field.clone(), json!(val_str));
                }

                let analytics_event = AnalyticsEvent {
                    event_type: stream.to_string(),
                    payload: serde_json::Value::Object(payload),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                if let Err(e) = index_event_to_opensearch(&analytics_event).await {
                    eprintln!("❌ Failed to send event to OpenSearch: {}", e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let streams = vec![
        "orders_stream",
        "inventory_stream",
        "suppliers_stream",
        "restaurants_stream",
        "payments_stream",
        "product_stream"
    ];

    for stream in streams {
        let s = stream.to_string();
        task::spawn(async move {
            consume_stream(&s).await;
        });
    }

    // Prevent the app from exiting
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
