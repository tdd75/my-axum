// Producer implementations
mod kafka_producer;
mod rabbitmq_producer;
mod redis_producer;

use async_trait::async_trait;

/// Generic message producer trait for publishing task events to different message brokers
/// Works with any task type T that is serializable
#[async_trait]
pub trait MessageProducer: Send + Sync {
    /// Publish a generic task event to the broker (serializes to JSON)
    async fn publish_event_json(
        &self,
        event_json: &str,
        destination: Option<&str>,
    ) -> anyhow::Result<()>;
}

/// Producer configuration enum
#[derive(Debug, Clone)]
pub enum ProducerConfig {
    Kafka {
        brokers: String,
        default_topic: String,
    },
    RabbitMQ {
        url: String,
        default_queue: String,
    },
    Redis {
        url: String,
        default_channel: String,
    },
}

impl ProducerConfig {
    /// Create Kafka producer configuration
    pub fn kafka(brokers: String, default_topic: String) -> Self {
        ProducerConfig::Kafka {
            brokers,
            default_topic,
        }
    }

    /// Create RabbitMQ producer configuration
    pub fn rabbitmq(url: String, default_queue: String) -> Self {
        ProducerConfig::RabbitMQ { url, default_queue }
    }

    /// Create Redis producer configuration
    pub fn redis(url: String, default_channel: String) -> Self {
        ProducerConfig::Redis {
            url,
            default_channel,
        }
    }
}

/// Create a message producer based on configuration (async version)
pub async fn create_producer(config: ProducerConfig) -> anyhow::Result<Box<dyn MessageProducer>> {
    match config {
        ProducerConfig::Kafka {
            brokers,
            default_topic,
        } => {
            use kafka_producer::KafkaProducer;
            Ok(Box::new(
                KafkaProducer::new(&brokers, &default_topic).await?,
            ))
        }
        ProducerConfig::RabbitMQ { url, default_queue } => {
            use rabbitmq_producer::RabbitMQProducer;
            Ok(Box::new(RabbitMQProducer::new(&url, &default_queue).await?))
        }
        ProducerConfig::Redis {
            url,
            default_channel,
        } => {
            use redis_producer::RedisProducer;
            Ok(Box::new(RedisProducer::new(&url, &default_channel).await?))
        }
    }
}
