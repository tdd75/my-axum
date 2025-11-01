use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use rdkafka::{
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
    message::Message,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

use crate::messaging::{MessageProducer, TaskEvent, TaskHandler, ensure_topics_exist};
use serde::{Deserialize, Serialize};

use super::MessageConsumer;
use super::task_queue::{
    SharedPriorityQueue, enqueue_task, new_priority_queue, spawn_priority_processor,
};

/// Kafka consumer for processing background tasks
pub struct KafkaConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    consumer: StreamConsumer,
    task_handler: Arc<dyn TaskHandler<T>>,
    semaphore: Arc<Semaphore>,
    priority_queue: SharedPriorityQueue<T>,
    producer: Arc<Box<dyn MessageProducer>>,
}

impl<T> KafkaConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    /// Create a new Kafka consumer
    pub async fn new(
        brokers: &str,
        group_id: &str,
        topics: &[String],
        task_handler: Arc<dyn TaskHandler<T>>,
        semaphore: Arc<Semaphore>,
        producer: Arc<Box<dyn MessageProducer>>,
    ) -> Result<Self> {
        // Ensure Kafka topics exist if using Kafka
        info!("Ensuring Kafka topics exist...");
        let topic_refs: Vec<&str> = topics.iter().map(|s| s.as_str()).collect();
        ensure_topics_exist(brokers, &topic_refs).await?;

        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .set("auto.commit.interval.ms", "5000")
            .set("enable.auto.offset.store", "false")
            .create()
            .context("Failed to create Kafka consumer")?;

        // Subscribe to topics
        let topic_refs: Vec<&str> = topics.iter().map(|s| s.as_str()).collect();
        consumer
            .subscribe(&topic_refs)
            .context("Failed to subscribe to topics")?;

        info!("Kafka consumer subscribed to topics: {:?}", topics);

        Ok(Self {
            consumer,
            task_handler,
            semaphore,
            priority_queue: new_priority_queue(),
            producer,
        })
    }

    /// Start consuming messages from Kafka
    async fn consume_messages(&mut self) -> anyhow::Result<()> {
        info!("Starting Kafka message consumption...");

        spawn_priority_processor(
            self.priority_queue.clone(),
            self.task_handler.clone(),
            self.semaphore.clone(),
            self.producer.clone(),
        );

        loop {
            match self.consumer.recv().await {
                Err(e) => {
                    error!("Kafka consumer error: {:?}", e);
                    if matches!(e, KafkaError::Global(_)) {
                        // Fatal error, break the loop
                        return Err(anyhow::anyhow!("Fatal Kafka error: {:?}", e));
                    }
                    // For non-fatal errors, continue consuming
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
                Ok(message) => {
                    // Get message payload
                    let payload = match message.payload() {
                        Some(p) => p,
                        None => {
                            warn!("Received message with no payload");
                            continue;
                        }
                    };

                    // Parse task event
                    let event: TaskEvent<T> = match serde_json::from_slice(payload) {
                        Ok(e) => e,
                        Err(e) => {
                            error!("Failed to parse task event: {:?}", e);
                            continue;
                        }
                    };

                    info!(
                        "Received task event: {} with priority {:?} from topic: {}",
                        event.id,
                        event.priority,
                        message.topic()
                    );

                    // Add to priority queue instead of processing immediately
                    enqueue_task(&self.priority_queue, event).await;
                }
            }
        }
    }
}

#[async_trait]
impl<T> MessageConsumer for KafkaConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    async fn connect(&mut self) -> anyhow::Result<()> {
        // Kafka connection is established in constructor
        Ok(())
    }

    async fn consume(&mut self) -> anyhow::Result<()> {
        self.consume_messages().await
    }

    fn broker_type(&self) -> &str {
        "Kafka"
    }

    async fn close(&mut self) -> anyhow::Result<()> {
        info!("Closing Kafka consumer");
        Ok(())
    }
}
