use deadpool_redis::{Config, Pool, Runtime};
use redis::{aio::MultiplexedConnection, Client, RedisError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone)]
pub struct StreamPublisher {
    pool: Option<Pool>,
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
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let cfg = Config::from_url(redis_url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(Self {
            pool: Some(pool),
            enabled: true,
        })
    }

    pub fn noop() -> Self {
        Self {
            pool: None,
            enabled: false,
        }
    }

    pub async fn publish<T: Serialize>(
        &self,
        event_type: &str,
        message: &T,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Ok("noop".to_string());
        }

        let stream = stream_for_event(event_type);
        let payload = serde_json::to_string(message)?;

        let Some(pool) = &self.pool else {
            return Ok("noop".to_string());
        };

        let mut conn = pool.get().await?;
        let res: String = redis::cmd("XADD")
            .arg(stream)
            .arg("*")
            .arg("event_type")
            .arg(event_type)
            .arg("payload")
            .arg(payload)
            .query_async(&mut *conn)
            .await?;
        Ok(res)
    }

    pub fn publish_async<T>(&self, event_type: &str, message: T)
    where
        T: Serialize + Send + Sync + 'static,
    {
        let this = self.clone();
        let event_type = event_type.to_string();
        tokio::spawn(async move {
            let error_str = match this.publish(&event_type, &message).await {
                Ok(_) => return,
                Err(e) => e.to_string(),
            };
            
            tracing::warn!(%event_type, error = %error_str, "redis stream publish failed, routing to DLQ");
            if let Some(pool) = &this.pool {
                if let Ok(mut conn) = pool.get().await {
                    let payload = serde_json::to_string(&message).unwrap_or_default();
                    let _: Result<(), _> = redis::cmd("XADD")
                        .arg("stream:dlq")
                        .arg("*")
                        .arg("event_type")
                        .arg(&event_type)
                        .arg("payload")
                        .arg(payload)
                        .arg("error")
                        .arg(&error_str)
                        .query_async(&mut *conn)
                        .await;
                }
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
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: DeserializeOwned + Send + 'static,
    F: FnMut(StreamEnvelope<T>) -> Fut,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>,
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
            if let Ok(()) = handler(event).await {
                let _: Result<(), RedisError> = redis::cmd("XACK")
                    .arg(stream)
                    .arg(group)
                    .arg(id)
                    .query_async(&mut conn)
                    .await;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct DummyEvent {
        value: i32,
    }

    #[test]
    fn test_stream_for_event() {
        assert_eq!(stream_for_event("product.created"), "stream:products");
        assert_eq!(stream_for_event("order.updated"), "stream:orders");
        assert_eq!(stream_for_event("inventory.reserved"), "stream:inventory");
        assert_eq!(stream_for_event("logistics.shipped"), "stream:logistics");
        assert_eq!(stream_for_event("payment.processed"), "stream:payments");
        assert_eq!(stream_for_event("user.registered"), "stream:users");
        assert_eq!(stream_for_event("supplier.created"), "stream:suppliers");
        assert_eq!(stream_for_event("tenant.updated"), "stream:suppliers");
        assert_eq!(stream_for_event("notification.sent"), "stream:notifications");
        assert_eq!(stream_for_event("unknown.event"), "stream:platform");
    }

    #[test]
    fn test_streams_for_events() {
        let events = vec!["product.created", "order.created", "product.updated"];
        let streams = streams_for_events(&events);
        assert_eq!(streams.len(), 2);
        assert!(streams.contains(&"stream:products"));
        assert!(streams.contains(&"stream:orders"));
    }

    #[test]
    fn test_stream_publisher_noop() {
        let publisher = StreamPublisher::noop();
        assert!(!publisher.enabled);
        assert!(publisher.pool.is_none());
    }

    #[tokio::test]
    async fn test_publish_noop() {
        let publisher = StreamPublisher::noop();
        let res = publisher.publish("order.created", &DummyEvent { value: 1 }).await.unwrap();
        assert_eq!(res, "noop");
    }

    #[test]
    fn test_value_to_string() {
        let b = b"hello";
        let data = redis::Value::Data(b.to_vec());
        assert_eq!(value_to_string(&data), "hello");

        let status = redis::Value::Status("good".to_string());
        assert_eq!(value_to_string(&status), "good");

        let ok = redis::Value::Okay;
        assert_eq!(value_to_string(&ok), "OK");

        let i = redis::Value::Int(42);
        assert_eq!(value_to_string(&i), "42");
    }

    #[test]
    fn test_parse_stream_reply() {
        // Construct a mock redis value for stream reply
        // [
        //   [ "stream:orders", [
        //       [ "123-0", ["event_type", "order.created", "payload", "{\"value\":42}"] ]
        //     ]
        //   ]
        // ]
        let message_fields = redis::Value::Bulk(vec![
            redis::Value::Data(b"event_type".to_vec()),
            redis::Value::Data(b"order.created".to_vec()),
            redis::Value::Data(b"payload".to_vec()),
            redis::Value::Data(b"{\"value\":42}".to_vec()),
        ]);

        let message = redis::Value::Bulk(vec![
            redis::Value::Data(b"123-0".to_vec()),
            message_fields,
        ]);

        let stream = redis::Value::Bulk(vec![
            redis::Value::Data(b"stream:orders".to_vec()),
            redis::Value::Bulk(vec![message]),
        ]);

        let reply = redis::Value::Bulk(vec![stream]);

        let events: Vec<StreamEnvelope<DummyEvent>> = parse_stream_reply(reply, &["order.created"]);
        
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].stream, "stream:orders");
        assert_eq!(events[0].id, "123-0");
        assert_eq!(events[0].event_type, "order.created");
        assert_eq!(events[0].payload.value, 42);
    }
}
