use async_trait::async_trait;
use futures::StreamExt;
use rdkafka::{
    Message,
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    client::DefaultClientContext,
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
};
use tracing::{error, info, warn};

use super::{MessageForwarder, forward_message_to_websocket};

/// Kafka consumer that forwards progress updates to WebSockets
pub struct KafkaForwarder {
    consumer: StreamConsumer,
    topic: String,
}

impl KafkaForwarder {
    /// Create a new Kafka forwarder
    pub async fn new(brokers: &str, topic: &str, consumer_group: &str) -> anyhow::Result<Self> {
        // Ensure topic exists
        Self::ensure_topic_exists(brokers, topic).await?;

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", consumer_group)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "latest")
            .create()?;

        consumer.subscribe(&[topic])?;
        info!("✓ Kafka forwarder initialized for topic: {}", topic);

        Ok(Self {
            consumer,
            topic: topic.to_string(),
        })
    }

    /// Ensure a Kafka topic exists, create it if it doesn't
    async fn ensure_topic_exists(brokers: &str, topic: &str) -> anyhow::Result<()> {
        info!("Checking if Kafka topic exists: {}", topic);

        let admin_client: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .create()?;

        let new_topic = NewTopic::new(
            topic,
            1,                          // num_partitions
            TopicReplication::Fixed(1), // replication_factor
        );

        let opts = AdminOptions::new();
        let results = admin_client.create_topics(&[new_topic], &opts).await?;

        match results.first() {
            Some(Ok(_)) => info!("✓ Topic '{}' created successfully", topic),
            Some(Err(e)) => {
                if format!("{:?}", e).contains("TopicAlreadyExists") {
                    info!("✓ Topic '{}' already exists", topic);
                } else {
                    warn!("Warning creating topic '{}': {:?}", topic, e);
                }
            }
            None => {}
        }

        Ok(())
    }
}

#[async_trait]
impl MessageForwarder for KafkaForwarder {
    async fn start_forwarding(self: Box<Self>) -> anyhow::Result<()> {
        let this = *self;
        info!("✓ Kafka forwarder subscribed to topic: {}", this.topic);

        let mut stream = this.consumer.stream();

        loop {
            match stream.next().await {
                Some(Ok(msg)) => {
                    if let Some(payload) = msg.payload_view::<str>() {
                        match payload {
                            Ok(p) => {
                                forward_message_to_websocket(p).await;
                            }
                            Err(e) => {
                                error!("Failed to decode Kafka message payload: {:?}", e);
                            }
                        }
                    }
                }
                Some(Err(e)) => {
                    error!("Kafka consumer error: {}", e);
                }
                None => {
                    error!("Kafka consumer stream ended");
                    break;
                }
            }
        }

        Ok(())
    }
}
