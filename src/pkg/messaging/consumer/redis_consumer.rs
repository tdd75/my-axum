use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use futures_util::StreamExt; // For stream.next().await
use redis::Client;
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

/// Redis consumer for processing background tasks
pub struct RedisConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    client: Client,
    channels: Vec<String>,
    task_handler: Arc<dyn TaskHandler<T>>,
    semaphore: Arc<Semaphore>,
    priority_queue: Arc<Mutex<BinaryHeap<PriorityTask<T>>>>,
    producer: Arc<Box<dyn MessageProducer>>,
}

impl<T> RedisConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    /// Create a new Redis consumer
    pub async fn new(
        redis_url: &str,
        channels: &[String],
        task_handler: Arc<dyn TaskHandler<T>>,
        semaphore: Arc<Semaphore>,
        producer: Arc<Box<dyn MessageProducer>>,
    ) -> Result<Self> {
        let client = Client::open(redis_url).context("Failed to create Redis client")?;

        info!("Redis consumer initialized for channels: {:?}", channels);

        Ok(Self {
            client,
            channels: channels.to_vec(),
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

    /// Internal consume implementation  
    async fn consume_messages(&mut self) -> anyhow::Result<()> {
        info!("🔄 Starting Redis Pub/Sub consumption...");

        let mut pubsub = self
            .client
            .get_async_pubsub()
            .await
            .context("Failed to get Redis pub/sub connection")?;

        // Subscribe to all channels
        for channel in &self.channels {
            pubsub
                .subscribe(channel)
                .await
                .context(format!("Failed to subscribe to channel: {}", channel))?;
            info!("✓ Subscribed to Redis channel: {}", channel);
        }

        info!("🎯 Redis consumer is now listening for messages...");

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

        let mut stream = pubsub.on_message();

        loop {
            match stream.next().await {
                Some(msg) => {
                    let payload: String = msg.get_payload()?;
                    let channel_name = msg.get_channel_name();

                    // Parse task event
                    let event: TaskEvent<T> = match serde_json::from_str(&payload) {
                        Ok(e) => e,
                        Err(e) => {
                            error!("Failed to parse task event from Redis: {:?}", e);
                            continue;
                        }
                    };

                    info!(
                        "Received task event: {} with priority {:?} from channel: {}",
                        event.id, event.priority, channel_name
                    );

                    // Add to priority queue instead of processing immediately
                    {
                        let mut queue = self.priority_queue.lock().await;
                        queue.push(PriorityTask { event });
                    }
                }
                None => {
                    warn!("Redis Pub/Sub stream ended");
                    break;
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<T> MessageConsumer for RedisConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    async fn connect(&mut self) -> anyhow::Result<()> {
        // Test connection by creating a pubsub connection
        let mut pubsub = self
            .client
            .get_async_pubsub()
            .await
            .context("Failed to connect to Redis for pub/sub")?;

        // Test subscription to verify connection
        pubsub
            .subscribe(&self.channels[0])
            .await
            .context("Failed to test subscription")?;
        pubsub.unsubscribe(&self.channels[0]).await.ok();

        info!("✓ Redis pub/sub connection verified");
        Ok(())
    }

    async fn consume(&mut self) -> anyhow::Result<()> {
        self.consume_messages().await
    }

    fn broker_type(&self) -> &str {
        "Redis"
    }

    async fn close(&mut self) -> anyhow::Result<()> {
        info!("Closing Redis consumer");
        Ok(())
    }
}
