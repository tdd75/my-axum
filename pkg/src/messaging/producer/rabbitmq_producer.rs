use anyhow::{Context, Result};
use async_trait::async_trait;
use lapin::{
    BasicProperties, Connection, ConnectionProperties,
    options::{BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
};

use super::{MessageProducer, event_id_from_json};

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

        let event_id = event_id_from_json(event_json);
        let channel = self
            .connection
            .create_channel()
            .await
            .context("Failed to create RabbitMQ channel")?;

        channel
            .queue_declare(
                queue.into(),
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .context("Failed to declare queue")?;

        channel
            .basic_publish(
                "".into(),
                queue.into(),
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
