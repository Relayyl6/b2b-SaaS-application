use redis::{aio::MultiplexedConnection, Client, RedisError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone)]
pub struct StreamPublisher {
    client: Option<Client>,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct StreamEnvelope<T> {
    pub stream: String,
    pub id: String,
    pub event_type: String,
    pub payload: T,
}

impl StreamPublisher {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        Ok(Self {
            client: Some(Client::open(redis_url)?),
            enabled: true,
        })
    }

    pub fn noop() -> Self {
        Self {
            client: None,
            enabled: false,
        }
    }

    pub async fn publish<T: Serialize>(
        &self,
        event_type: &str,
        message: &T,
    ) -> Result<String, RedisError> {
        if !self.enabled {
            return Ok("noop".to_string());
        }

        let stream = stream_for_event(event_type);
        let payload = serde_json::to_string(message).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "event serialization failed",
                e.to_string(),
            ))
        })?;

        let Some(client) = &self.client else {
            return Ok("noop".to_string());
        };

        let mut conn = client.get_multiplexed_async_connection().await?;
        redis::cmd("XADD")
            .arg(stream)
            .arg("*")
            .arg("event_type")
            .arg(event_type)
            .arg("payload")
            .arg(payload)
            .query_async(&mut conn)
            .await
    }

    pub fn publish_async<T>(&self, event_type: &str, message: T)
    where
        T: Serialize + Send + Sync + 'static,
    {
        let this = self.clone();
        let event_type = event_type.to_string();
        tokio::spawn(async move {
            if let Err(e) = this.publish(&event_type, &message).await {
                tracing::warn!(%event_type, error = ?e, "redis stream publish failed");
            }
        });
    }
}

pub fn stream_for_event(event_type: &str) -> &'static str {
    match event_type.split('.').next().unwrap_or("platform") {
        "product" => "stream:products",
        "order" => "stream:orders",
        "inventory" => "stream:inventory",
        "logistics" => "stream:logistics",
        "payment" => "stream:payments",
        "user" => "stream:users",
        "supplier" | "tenant" => "stream:suppliers",
        "notification" => "stream:notifications",
        _ => "stream:platform",
    }
}

pub fn streams_for_events(events: &[&str]) -> Vec<&'static str> {
    let mut streams = Vec::new();
    for event in events {
        let stream = stream_for_event(event);
        if !streams.contains(&stream) {
            streams.push(stream);
        }
    }
    streams
}

pub async fn ensure_consumer_group(conn: &mut MultiplexedConnection, stream: &str, group: &str) {
    let _: Result<(), RedisError> = redis::cmd("XGROUP")
        .arg("CREATE")
        .arg(stream)
        .arg(group)
        .arg("0")
        .arg("MKSTREAM")
        .query_async(conn)
        .await;
}

pub async fn consume_json<T, F, Fut>(
    redis_url: &str,
    group: &str,
    consumer: &str,
    event_types: &[&str],
    mut handler: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: DeserializeOwned + Send + 'static,
    F: FnMut(StreamEnvelope<T>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let streams = streams_for_events(event_types);
    let client = Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;

    for stream in &streams {
        ensure_consumer_group(&mut conn, stream, group).await;
    }

    loop {
        let mut cmd = redis::cmd("XREADGROUP");
        cmd.arg("GROUP")
            .arg(group)
            .arg(consumer)
            .arg("BLOCK")
            .arg(5000)
            .arg("COUNT")
            .arg(50)
            .arg("STREAMS");
        for stream in &streams {
            cmd.arg(stream);
        }
        for _ in &streams {
            cmd.arg(">");
        }

        let reply: redis::Value = cmd.query_async(&mut conn).await?;
        let events = parse_stream_reply::<T>(reply, event_types);

        for event in events {
            let stream = event.stream.clone();
            let id = event.id.clone();
            handler(event).await;
            let _: Result<(), RedisError> = redis::cmd("XACK")
                .arg(stream)
                .arg(group)
                .arg(id)
                .query_async(&mut conn)
                .await;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn parse_stream_reply<T: DeserializeOwned>(
    reply: redis::Value,
    event_types: &[&str],
) -> Vec<StreamEnvelope<T>> {
    let mut output = Vec::new();
    let redis::Value::Bulk(streams) = reply else {
        return output;
    };

    for stream_value in streams {
        let redis::Value::Bulk(parts) = stream_value else {
            continue;
        };
        if parts.len() != 2 {
            continue;
        }
        let stream = value_to_string(&parts[0]);
        let redis::Value::Bulk(messages) = &parts[1] else {
            continue;
        };

        for message in messages {
            let redis::Value::Bulk(message_parts) = message else {
                continue;
            };
            if message_parts.len() != 2 {
                continue;
            }
            let id = value_to_string(&message_parts[0]);
            let redis::Value::Bulk(fields) = &message_parts[1] else {
                continue;
            };

            let mut map = HashMap::new();
            let mut iter = fields.iter();
            while let (Some(key), Some(value)) = (iter.next(), iter.next()) {
                map.insert(value_to_string(key), value_to_string(value));
            }

            let Some(event_type) = map.get("event_type").cloned() else {
                continue;
            };
            if !event_types.contains(&event_type.as_str()) {
                continue;
            }
            let Some(payload) = map.get("payload") else {
                continue;
            };
            let Ok(payload) = serde_json::from_str::<T>(payload) else {
                continue;
            };

            output.push(StreamEnvelope {
                stream: stream.clone(),
                id,
                event_type,
                payload,
            });
        }
    }

    output
}

fn value_to_string(value: &redis::Value) -> String {
    match value {
        redis::Value::Data(bytes) => String::from_utf8_lossy(bytes).to_string(),
        redis::Value::Status(value) => value.clone(),
        redis::Value::Okay => "OK".to_string(),
        redis::Value::Int(value) => value.to_string(),
        _ => String::new(),
    }
}
