mod kafka_forwarder;
mod rabbitmq_forwarder;
mod redis_forwarder;

pub use kafka_forwarder::KafkaForwarder;
pub use rabbitmq_forwarder::RabbitMQForwarder;
pub use redis_forwarder::RedisForwarder;

use super::websocket::{BroadcastMessage, broadcast_to_task, broadcast_to_user};
use async_trait::async_trait;
use tracing::{debug, error};

/// Configuration for creating message forwarders
#[derive(Debug, Clone)]
pub enum ForwarderConfig {
    Kafka {
        brokers: String,
        consumer_group: String,
        topic: String,
    },
    Redis {
        url: String,
        channel: String,
    },
    RabbitMQ {
        url: String,
        queue: String,
    },
}

impl ForwarderConfig {
    /// Create Kafka forwarder configuration
    pub fn kafka(brokers: String, topic: String, consumer_group: String) -> Self {
        ForwarderConfig::Kafka {
            brokers,
            consumer_group,
            topic,
        }
    }

    /// Create Redis forwarder configuration
    pub fn redis(url: String, channel: String) -> Self {
        ForwarderConfig::Redis { url, channel }
    }

    /// Create RabbitMQ forwarder configuration
    pub fn rabbitmq(url: String, queue: String) -> Self {
        ForwarderConfig::RabbitMQ { url, queue }
    }
}

/// Trait for message forwarders that receive broadcast messages from a message queue
/// and forward them to WebSocket connections
#[async_trait]
pub trait MessageForwarder: Send + Sync {
    /// Start forwarding messages from the message queue to WebSockets
    async fn start_forwarding(self: Box<Self>) -> anyhow::Result<()>;
}

/// Helper function to process and forward a broadcast message to the appropriate task
pub async fn forward_message_to_websocket(payload: &str) {
    // Try to parse as a broadcast message
    let broadcast_msg: BroadcastMessage = match serde_json::from_str(payload) {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to parse broadcast message: {}", e);
            return;
        }
    };

    // Try to extract task_id first (general purpose), fall back to user_id (backward compatibility)
    if let Some(task_id) = broadcast_msg.data.get("task_id").and_then(|v| v.as_str()) {
        debug!(
            "Forwarding {} to task {}",
            broadcast_msg.event_type, task_id
        );
        let task_id = task_id.to_string();
        broadcast_to_task(&task_id, broadcast_msg).await;
    } else if let Some(user_id) = broadcast_msg.data.get("user_id").and_then(|v| v.as_i64()) {
        debug!(
            "Forwarding {} to user {} (backward compatibility)",
            broadcast_msg.event_type, user_id
        );
        broadcast_to_user(user_id as i32, broadcast_msg).await;
    } else {
        error!(
            "Progress message missing task_id or user_id: {:?}",
            broadcast_msg
        );
    }
}

/// Create a message forwarder based on the configuration
pub async fn create_forwarder(
    config: ForwarderConfig,
) -> anyhow::Result<Box<dyn MessageForwarder>> {
    match config {
        ForwarderConfig::Redis { url, channel } => {
            let forwarder = RedisForwarder::new(&url, &channel).await?;
            Ok(Box::new(forwarder))
        }
        ForwarderConfig::Kafka {
            brokers,
            consumer_group,
            topic,
        } => {
            let forwarder =
                KafkaForwarder::new(&brokers, &topic, &format!("{}_forwarder", consumer_group))
                    .await?;
            Ok(Box::new(forwarder))
        }
        ForwarderConfig::RabbitMQ { url, queue } => {
            let forwarder = RabbitMQForwarder::new(&url, &queue).await?;
            Ok(Box::new(forwarder))
        }
    }
}
