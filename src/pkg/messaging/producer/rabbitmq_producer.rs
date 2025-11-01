use anyhow::{Context, Result};
use async_trait::async_trait;
use lapin::{
    BasicProperties, Connection, ConnectionProperties,
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
};

use super::MessageProducer;

/// RabbitMQ producer implementation
pub struct RabbitMQProducer {
    connection: Connection,
    default_queue: String,
}

impl RabbitMQProducer {
    pub async fn new(url: &str, default_queue: &str) -> Result<Self> {
        let connection = Connection::connect(url, ConnectionProperties::default())
            .await
            .context("Failed to connect to RabbitMQ")?;

        Ok(Self {
            connection,
            default_queue: default_queue.to_string(),
        })
    }
}

#[async_trait]
impl MessageProducer for RabbitMQProducer {
    async fn publish_event_json(
        &self,
        event_json: &str,
        destination: Option<&str>,
    ) -> anyhow::Result<()> {
        let queue = destination.unwrap_or(&self.default_queue);

        // Extract event ID from JSON for logging
        let event_id = serde_json::from_str::<serde_json::Value>(event_json)
            .ok()
            .and_then(|v| v.get("id").and_then(|id| id.as_str()).map(String::from))
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Create channel
        let channel = self
            .connection
            .create_channel()
            .await
            .context("Failed to create RabbitMQ channel")?;

        // Declare queue (idempotent)
        channel
            .queue_declare(
                queue,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .context("Failed to declare queue")?;

        // Publish message
        channel
            .basic_publish(
                "",
                queue,
                BasicPublishOptions::default(),
                event_json.as_bytes(),
                BasicProperties::default()
                    .with_delivery_mode(2) // Persistent
                    .with_content_type("application/json".into()),
            )
            .await
            .context("Failed to publish message to RabbitMQ")?
            .await
            .context("Failed to confirm message publish to RabbitMQ")?;

        tracing::info!(
            "✓ Published task event {} to RabbitMQ queue: {}",
            event_id,
            queue
        );
        Ok(())
    }
}
