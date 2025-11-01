use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use futures_util::StreamExt;
use lapin::{
    Channel, Connection, ConnectionProperties, message::Delivery, options::*, types::FieldTable,
};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{error, info};

use crate::messaging::{MessageProducer, TaskEvent, TaskHandler};
use serde::{Deserialize, Serialize};

use super::MessageConsumer;
use super::task_queue::{
    SharedPriorityQueue, enqueue_task, new_priority_queue, spawn_priority_processor,
};

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
    priority_queue: SharedPriorityQueue<T>,
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
            priority_queue: new_priority_queue(),
            producer,
        })
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
                    queue_name.as_str().into(),
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

        spawn_priority_processor(
            self.priority_queue.clone(),
            self.task_handler.clone(),
            self.semaphore.clone(),
            self.producer.clone(),
        );

        // Create consumers for all queues
        let mut handles = vec![];

        for queue_name in &self.queues {
            let queue = queue_name.clone();
            let consumer = channel
                .basic_consume(
                    queue.as_str().into(),
                    format!("consumer-{}", queue).into(),
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
        priority_queue: SharedPriorityQueue<T>,
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
        enqueue_task(&priority_queue, event).await;

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
            channel.close(0, "Normal shutdown".into()).await?;
        }
        if let Some(connection) = &self.connection {
            connection.close(0, "Normal shutdown".into()).await?;
        }
        Ok(())
    }
}
