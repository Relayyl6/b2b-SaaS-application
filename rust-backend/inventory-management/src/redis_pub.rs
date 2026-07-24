use platform::streams::StreamPublisher;

#[derive(Clone)]
pub struct RedisPublisher {
    publisher: StreamPublisher,
}

impl RedisPublisher {
    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            publisher: StreamPublisher::new(redis_url)?,
        })
    }

    pub async fn publish<T: serde::Serialize>(
        &self,
        event_type: &str,
        message: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.publisher.publish(event_type, message).await?;
        Ok(())
    }

    pub fn publish_async<T>(&self, event_type: &str, message: T)
    where
        T: serde::Serialize + Send + Sync + 'static,
    {
        self.publisher.publish_async(event_type, message);
    }

    pub fn new_noop() -> Self {
        Self {
            publisher: StreamPublisher::noop(),
        }
    }
}
