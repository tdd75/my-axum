use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use rdkafka::{
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
    message::Message,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tracing::{error, info, warn};

use crate::pkg::messaging::{MessageProducer, TaskEvent, TaskHandler, ensure_topics_exist};
use serde::{Deserialize, Serialize};

use super::MessageConsumer;

/// Wrapper for TaskEvent to implement Ord for priority queue
#[derive(Clone)]
struct PriorityTask<T>
where
    T: Clone + Send + Sync,
{
    event: TaskEvent<T>,
}

impl<T> PartialEq for PriorityTask<T>
where
    T: Clone + Send + Sync,
{
    fn eq(&self, other: &Self) -> bool {
        self.event.priority == other.event.priority
            && self.event.created_at == other.event.created_at
    }
}

impl<T> Eq for PriorityTask<T> where T: Clone + Send + Sync {}

impl<T> PartialOrd for PriorityTask<T>
where
    T: Clone + Send + Sync,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for PriorityTask<T>
where
    T: Clone + Send + Sync,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier created_at
        match self.event.priority.cmp(&other.event.priority) {
            Ordering::Equal => other.event.created_at.cmp(&self.event.created_at),
            other_order => other_order,
        }
    }
}

/// Kafka consumer for processing background tasks
pub struct KafkaConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    consumer: StreamConsumer,
    task_handler: Arc<dyn TaskHandler<T>>,
    semaphore: Arc<Semaphore>,
    priority_queue: Arc<Mutex<BinaryHeap<PriorityTask<T>>>>,
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
            priority_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            producer,
        })
    }

    /// Run priority queue processor (static method)
    async fn run_priority_processor(
        priority_queue: Arc<Mutex<BinaryHeap<PriorityTask<T>>>>,
        task_handler: Arc<dyn TaskHandler<T>>,
        semaphore: Arc<Semaphore>,
        producer: Arc<Box<dyn MessageProducer>>,
    ) where
        T: 'static,
    {
        loop {
            // Check if we have tasks in the priority queue
            let task = {
                let mut queue = priority_queue.lock().await;
                queue.pop()
            };

            match task {
                Some(priority_task) => {
                    let event = priority_task.event;
                    info!(
                        "Processing task {} with priority {:?}",
                        event.id, event.priority
                    );

                    // Acquire permit from semaphore for worker pool
                    let permit = match semaphore.clone().acquire_owned().await {
                        Ok(p) => p,
                        Err(e) => {
                            error!("Failed to acquire semaphore: {:?}", e);
                            continue;
                        }
                    };
                    let handler = task_handler.clone();
                    let producer_clone = producer.clone();

                    // Spawn task to process task
                    tokio::spawn(async move {
                        let _permit = permit; // Hold permit until task completes

                        match handler.handle_task(&event).await {
                            Ok(_) => {
                                info!("Task {} completed successfully", event.id);
                            }
                            Err(e) => {
                                error!("Task {} failed: {:?}", event.id, e);
                                if event.should_retry() {
                                    warn!(
                                        "Task {} will be retried (attempt {}/{})",
                                        event.id,
                                        event.retry_count + 1,
                                        event.max_retries
                                    );

                                    // Increment retry count and republish
                                    let mut retry_event = event.clone();
                                    retry_event.increment_retry();

                                    // Calculate exponential backoff delay: 2^retry_count seconds
                                    let delay_secs = 2_u64.pow(retry_event.retry_count);
                                    info!(
                                        "Task {} will retry in {} seconds",
                                        retry_event.id, delay_secs
                                    );

                                    // Schedule retry with delay
                                    let producer_retry = producer_clone.clone();
                                    tokio::spawn(async move {
                                        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
                                        match retry_event
                                            .publish_with_producer(
                                                producer_retry.as_ref().as_ref(),
                                                None,
                                            )
                                            .await
                                        {
                                            Ok(_) => info!(
                                                "Task {} republished for retry",
                                                retry_event.id
                                            ),
                                            Err(e) => error!(
                                                "Failed to republish task {}: {:?}",
                                                retry_event.id, e
                                            ),
                                        }
                                    });
                                } else {
                                    error!(
                                        "Task {} exceeded max retries ({})",
                                        event.id, event.max_retries
                                    );
                                }
                            }
                        }
                    });
                }
                None => {
                    // No tasks in queue, wait a bit
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Start consuming messages from Kafka
    pub async fn consume(&mut self) -> anyhow::Result<()> {
        info!("Starting Kafka message consumption...");

        // Spawn task processor
        let processor_queue = self.priority_queue.clone();
        let processor_handler = self.task_handler.clone();
        let processor_semaphore = self.semaphore.clone();
        let processor_producer = self.producer.clone();
        tokio::spawn(async move {
            Self::run_priority_processor(
                processor_queue,
                processor_handler,
                processor_semaphore,
                processor_producer,
            )
            .await
        });

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
                    {
                        let mut queue = self.priority_queue.lock().await;
                        queue.push(PriorityTask { event });
                    }
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
        self.consume().await
    }

    fn broker_type(&self) -> &str {
        "Kafka"
    }

    async fn close(&mut self) -> anyhow::Result<()> {
        info!("Closing Kafka consumer");
        Ok(())
    }
}
