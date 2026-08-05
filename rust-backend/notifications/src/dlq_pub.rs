use crate::models::Notification;
use lapin::{
    options::{BasicPublishOptions, ConfirmSelectOptions, ExchangeDeclareOptions, QueueDeclareOptions, QueueBindOptions},
    types::{FieldTable, AMQPValue},
    BasicProperties, Connection, ConnectionProperties,
};
use std::env;
use tokio::time::{timeout, Duration};
use tracing::error;

#[derive(Clone)]
pub struct DlqPublisher {
    channel: Option<lapin::Channel>,
}

impl DlqPublisher {
    pub async fn new() -> Self {
        let amqp_addr = env::var("AMQP_ADDR")
            .unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into());

        match Connection::connect(&amqp_addr, ConnectionProperties::default()).await {
            Ok(conn) => {
                if let Ok(channel) = conn.create_channel().await {
                    let _ = channel.confirm_select(ConfirmSelectOptions::default()).await;
                    
                    let _ = channel.exchange_declare(
                        "notifications_dlx",
                        lapin::ExchangeKind::Direct,
                        ExchangeDeclareOptions { durable: true, ..Default::default() },
                        FieldTable::default(),
                    ).await;

                    let mut args = FieldTable::default();
                    args.insert("x-dead-letter-exchange".into(), AMQPValue::LongString("notifications_dlx".into()));
                    args.insert("x-message-ttl".into(), AMQPValue::LongInt(60000)); // 60 seconds delay

                    let _ = channel.queue_declare(
                        "notifications_retry_queue",
                        QueueDeclareOptions { durable: true, ..Default::default() },
                        args,
                    ).await;

                    let _ = channel.queue_bind(
                        "notifications_retry_queue",
                        "notifications_dlx",
                        "retry",
                        QueueBindOptions::default(),
                        FieldTable::default(),
                    ).await;

                    Self { channel: Some(channel) }
                } else {
                    Self { channel: None }
                }
            }
            Err(_) => Self { channel: None },
        }
    }

    pub async fn publish_to_dlq(&self, notification: &Notification, error: &str) {
        let Some(channel) = &self.channel else { return };
        
        let mut headers = FieldTable::default();
        headers.insert("error_reason".into(), AMQPValue::LongString(error.into()));
        headers.insert("notification_id".into(), AMQPValue::LongString(notification.id.to_string().into()));

        let payload = serde_json::to_vec(notification).unwrap_or_default();
        let props = BasicProperties::default().with_delivery_mode(2).with_headers(headers);

        let future = channel.basic_publish(
            "notifications_dlx",
            "retry",
            BasicPublishOptions::default(),
            &payload,
            props,
        );

        if let Err(e) = timeout(Duration::from_millis(500), future).await {
            error!("Failed to publish to DLQ: {}", e);
        }
    }
}
