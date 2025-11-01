// Consumer implementations
mod kafka_consumer;
mod rabbitmq_consumer;
mod redis_consumer;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::pkg::messaging::{MessageProducer, TaskHandler};

/// Message consumer abstraction trait
/// Allows worker to support different message brokers (Kafka, Redis, RabbitMQ)
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    /// Initialize the message broker connection
    async fn connect(&mut self) -> anyhow::Result<()>;

    /// Start consuming messages and process them
    async fn consume(&mut self) -> anyhow::Result<()>;

    /// Get broker type name for logging
    fn broker_type(&self) -> &str;

    /// Close connection gracefully
    async fn close(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Configuration for message consumer
#[derive(Debug, Clone)]
pub enum ConsumerConfig {
    Kafka {
        brokers: String,
        consumer_group: String,
        topics: Vec<String>,
    },
    Redis {
        url: String,
        channels: Vec<String>,
    },
    RabbitMQ {
        url: String,
        queues: Vec<String>,
    },
}

impl ConsumerConfig {
    /// Create Kafka consumer configuration
    pub fn kafka(brokers: String, consumer_group: String, topics: Vec<String>) -> Self {
        ConsumerConfig::Kafka {
            brokers,
            consumer_group,
            topics,
        }
    }

    /// Create Redis consumer configuration
    pub fn redis(url: String, channels: Vec<String>) -> Self {
        ConsumerConfig::Redis { url, channels }
    }

    /// Create RabbitMQ consumer configuration
    pub fn rabbitmq(url: String, queues: Vec<String>) -> Self {
        ConsumerConfig::RabbitMQ { url, queues }
    }
}

/// Create a message consumer instance based on configuration
pub async fn create_consumer<T>(
    config: ConsumerConfig,
    task_handler: Arc<dyn TaskHandler<T>>,
    semaphore: Arc<Semaphore>,
    producer: Arc<Box<dyn MessageProducer>>,
) -> anyhow::Result<Box<dyn MessageConsumer>>
where
    T: Clone + Send + Sync + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    match config {
        ConsumerConfig::Kafka {
            brokers,
            consumer_group,
            topics,
        } => {
            use kafka_consumer::KafkaConsumer;
            let consumer = KafkaConsumer::new(
                &brokers,
                &consumer_group,
                &topics,
                task_handler,
                semaphore,
                producer,
            )
            .await?;
            Ok(Box::new(consumer))
        }
        ConsumerConfig::Redis { url, channels } => {
            use redis_consumer::RedisConsumer;
            let consumer =
                RedisConsumer::new(&url, &channels, task_handler, semaphore, producer).await?;
            Ok(Box::new(consumer))
        }
        ConsumerConfig::RabbitMQ { url, queues } => {
            use rabbitmq_consumer::RabbitMQConsumer;
            let consumer =
                RabbitMQConsumer::new(&url, &queues, task_handler, semaphore, producer).await?;
            Ok(Box::new(consumer))
        }
    }
}
