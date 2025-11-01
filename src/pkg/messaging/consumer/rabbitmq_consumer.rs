use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use lapin::{
    Channel, Connection, ConnectionProperties, message::Delivery, options::*, types::FieldTable,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tracing::{error, info, warn};

use crate::pkg::messaging::{MessageProducer, TaskEvent, TaskHandler};
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

/// RabbitMQ consumer for processing background tasks
pub struct RabbitMQConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    url: String,
    queues: Vec<String>,
    task_handler: Arc<dyn TaskHandler<T>>,
    semaphore: Arc<Semaphore>,
    connection: Option<Connection>,
    channel: Option<Channel>,
    priority_queue: Arc<Mutex<BinaryHeap<PriorityTask<T>>>>,
    producer: Arc<Box<dyn MessageProducer>>,
}

impl<T> RabbitMQConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    /// Create a new RabbitMQ consumer
    pub async fn new(
        url: &str,
        queues: &[String],
        task_handler: Arc<dyn TaskHandler<T>>,
        semaphore: Arc<Semaphore>,
        producer: Arc<Box<dyn MessageProducer>>,
    ) -> Result<Self> {
        info!("RabbitMQ consumer initialized for queues: {:?}", queues);

        Ok(Self {
            url: url.to_string(),
            queues: queues.to_vec(),
            task_handler,
            semaphore,
            connection: None,
            channel: None,
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

    /// Internal consume implementation
    async fn consume_messages(&mut self) -> anyhow::Result<()> {
        info!("🔄 Starting RabbitMQ message consumption...");

        let channel = self
            .channel
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Channel not initialized. Call connect() first."))?;

        // Declare and consume from all queues
        for queue_name in &self.queues {
            // Declare queue (idempotent)
            channel
                .queue_declare(
                    queue_name,
                    QueueDeclareOptions {
                        durable: true,
                        ..Default::default()
                    },
                    FieldTable::default(),
                )
                .await
                .context(format!("Failed to declare queue: {}", queue_name))?;

            info!("✓ Queue '{}' declared", queue_name);
        }

        // Set QoS - prefetch count
        channel.basic_qos(10, BasicQosOptions::default()).await?;

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

        // Create consumers for all queues
        let mut handles = vec![];

        for queue_name in &self.queues {
            let queue = queue_name.clone();
            let consumer = channel
                .basic_consume(
                    &queue,
                    &format!("consumer-{}", queue),
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .context(format!("Failed to start consumer for queue: {}", queue))?;

            info!("✓ Started consuming from queue: {}", queue);

            let priority_queue = self.priority_queue.clone();
            let channel_clone = channel.clone();

            let handle = tokio::spawn(async move {
                let mut consumer = consumer;
                while let Some(delivery) = consumer.next().await {
                    match delivery {
                        Ok(delivery) => {
                            if let Err(e) = Self::process_delivery(
                                delivery,
                                priority_queue.clone(),
                                channel_clone.clone(),
                            )
                            .await
                            {
                                error!("Failed to process delivery: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!("Consumer error: {:?}", e);
                            break;
                        }
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all consumers
        for handle in handles {
            handle.await?;
        }

        info!("🎯 RabbitMQ consumer is now listening for messages...");

        Ok(())
    }

    async fn process_delivery(
        delivery: Delivery,
        priority_queue: Arc<Mutex<BinaryHeap<PriorityTask<T>>>>,
        _channel: Channel,
    ) -> anyhow::Result<()> {
        let payload = String::from_utf8(delivery.data.clone())?;

        // Parse task event
        let event: TaskEvent<T> = match serde_json::from_str(&payload) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to parse task event from RabbitMQ: {:?}", e);
                // Reject and don't requeue malformed messages
                delivery
                    .reject(BasicRejectOptions { requeue: false })
                    .await?;
                return Ok(());
            }
        };

        info!(
            "Received task event: {} with priority {:?}",
            event.id, event.priority
        );

        // Add to priority queue
        {
            let mut queue = priority_queue.lock().await;
            queue.push(PriorityTask { event });
        }

        // Acknowledge message immediately since we've queued it
        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    }
}

#[async_trait]
impl<T> MessageConsumer for RabbitMQConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    async fn connect(&mut self) -> anyhow::Result<()> {
        info!("🔌 Connecting to RabbitMQ at: {}", self.url);

        let connection = Connection::connect(&self.url, ConnectionProperties::default())
            .await
            .context("Failed to connect to RabbitMQ")?;

        let channel = connection
            .create_channel()
            .await
            .context("Failed to create RabbitMQ channel")?;

        info!("✓ RabbitMQ connection established");

        self.connection = Some(connection);
        self.channel = Some(channel);

        Ok(())
    }

    async fn consume(&mut self) -> anyhow::Result<()> {
        self.consume_messages().await
    }

    fn broker_type(&self) -> &str {
        "RabbitMQ"
    }

    async fn close(&mut self) -> anyhow::Result<()> {
        info!("Closing RabbitMQ consumer");
        if let Some(channel) = &self.channel {
            channel.close(0, "Normal shutdown").await?;
        }
        if let Some(connection) = &self.connection {
            connection.close(0, "Normal shutdown").await?;
        }
        Ok(())
    }
}
