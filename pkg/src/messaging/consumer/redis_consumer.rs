use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use futures_util::StreamExt; // For stream.next().await
use redis::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

use crate::messaging::{MessageProducer, TaskEvent, TaskHandler};
use serde::{Deserialize, Serialize};

use super::MessageConsumer;
use super::task_queue::{
    SharedPriorityQueue, enqueue_task, new_priority_queue, spawn_priority_processor,
};

/// Redis consumer for processing background tasks
pub struct RedisConsumer<T>
where
    T: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    client: Client,
    channels: Vec<String>,
    task_handler: Arc<dyn TaskHandler<T>>,
    semaphore: Arc<Semaphore>,
    priority_queue: SharedPriorityQueue<T>,
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
            priority_queue: new_priority_queue(),
            producer,
        })
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
        spawn_priority_processor(
            self.priority_queue.clone(),
            self.task_handler.clone(),
            self.semaphore.clone(),
            self.producer.clone(),
        );

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
                    enqueue_task(&self.priority_queue, event).await;
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
