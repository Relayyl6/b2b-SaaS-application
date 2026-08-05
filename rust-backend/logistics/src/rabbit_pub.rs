use crate::models::LogisticsEvent;
use lapin::{
    options::{BasicPublishOptions, ConfirmSelectOptions, ExchangeDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties,
};
use tokio::time::{timeout, Duration};
use tracing::warn;

const RABBIT_TIMEOUT_MS: u64 = 500;

#[derive(Clone)]
pub struct RabbitPublisher {
    channel: lapin::Channel,
}

impl RabbitPublisher {
    pub async fn new(amqp_addr: &str) -> Result<Self, lapin::Error> {
        let conn = Connection::connect(amqp_addr, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;
        channel
            .confirm_select(ConfirmSelectOptions::default())
            .await?;
        channel
            .exchange_declare(
                "analytics_events_topic",
                lapin::ExchangeKind::Topic,
                ExchangeDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;
        Ok(Self { channel })
    }

    /// Publishes logistics events to the analytics topic exchange.
    pub async fn publish_event(&self, event: &LogisticsEvent) -> Result<(), lapin::Error> {
        let future = async {
            let payload = serde_json::to_vec(event)
                .map_err(|e| lapin::Error::from(std::io::Error::other(e.to_string())))?;
            let mut headers = FieldTable::default();
            headers.insert(
                "x-tenant-id".into(),
                lapin::types::AMQPValue::LongString(event.tenant_id.to_string().into()),
            );
            let confirm = self.channel
                .basic_publish(
                    "analytics_events_topic",
                    &event.event_type,
                    BasicPublishOptions::default(),
                    &payload,
                    BasicProperties::default().with_delivery_mode(2).with_headers(headers),
                )
                .await?;
            confirm.await?;
            Ok::<(), lapin::Error>(())
        };

        match timeout(Duration::from_millis(RABBIT_TIMEOUT_MS), future).await {
            Ok(result) => result,
            Err(_) => {
                warn!("rabbit publish timed out and was skipped");
                Ok(())
            }
        }
    }

    /// Publishes in detached mode to keep request paths non-blocking.
    pub fn publish_async(&self, event: LogisticsEvent) {
        let this = self.clone();
        tokio::spawn(async move {
            if let Err(err) = this.publish_event(&event).await {
                warn!(error = ?err, "rabbit publish failed");
            }
        });
    }
}
