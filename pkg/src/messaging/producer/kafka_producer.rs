use anyhow::{Context, Result};
use async_trait::async_trait;
use rdkafka::{
    ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
use std::time::Duration;

use super::{MessageProducer, event_id_from_json};

/// Kafka producer implementation
pub struct KafkaProducer {
    producer: FutureProducer,
    default_topic: String,
}

impl KafkaProducer {
    /// Create a new Kafka producer
    pub async fn new(brokers: &str, default_topic: &str) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("compression.type", "lz4")
            .create()
            .context("Failed to create Kafka producer")?;

        Ok(Self {
            producer,
            default_topic: default_topic.to_string(),
        })
    }
}

#[async_trait]
impl MessageProducer for KafkaProducer {
    async fn publish_event_json(
        &self,
        event_json: &str,
        destination: Option<&str>,
    ) -> anyhow::Result<()> {
        let topic = destination.unwrap_or(&self.default_topic);

        let event_id = event_id_from_json(event_json);
        let record = FutureRecord::to(topic).key("default").payload(event_json);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| anyhow::anyhow!("Failed to send message to Kafka: {:?}", err))?;

        tracing::info!("Published task event {} to Kafka topic {}", event_id, topic);
        Ok(())
    }
}
