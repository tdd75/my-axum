mod kafka_forwarder;
mod rabbitmq_forwarder;
mod redis_forwarder;

pub use kafka_forwarder::KafkaForwarder;
pub use rabbitmq_forwarder::RabbitMQForwarder;
pub use redis_forwarder::RedisForwarder;

use super::websocket::{BroadcastMessage, broadcast_to_task, broadcast_to_user};
use async_trait::async_trait;
use tokio::sync::watch;
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

pub type ShutdownSignal = watch::Receiver<bool>;

/// Trait for message forwarders that receive broadcast messages from a message queue
/// and forward them to WebSocket connections
#[async_trait]
pub trait MessageForwarder: Send + Sync {
    /// Start forwarding messages from the message queue to WebSockets
    async fn start_forwarding(self: Box<Self>, shutdown: ShutdownSignal) -> anyhow::Result<()>;
}

enum WebSocketTarget {
    Task(String),
    User(i32),
}

fn websocket_target(message: &BroadcastMessage) -> Option<WebSocketTarget> {
    message
        .data
        .get("task_id")
        .and_then(|value| value.as_str())
        .map(|task_id| WebSocketTarget::Task(task_id.to_string()))
        .or_else(|| {
            message
                .data
                .get("user_id")
                .and_then(|value| value.as_i64())
                .map(|user_id| WebSocketTarget::User(user_id as i32))
        })
}

/// Helper function to process and forward a broadcast message to the appropriate task
pub async fn forward_message_to_websocket(payload: &str) {
    let broadcast_msg: BroadcastMessage = match serde_json::from_str(payload) {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to parse broadcast message: {}", e);
            return;
        }
    };

    match websocket_target(&broadcast_msg) {
        Some(WebSocketTarget::Task(task_id)) => {
            debug!(
                "Forwarding {} to task {}",
                broadcast_msg.event_type, task_id
            );
            broadcast_to_task(&task_id, broadcast_msg).await;
        }
        Some(WebSocketTarget::User(user_id)) => {
            debug!(
                "Forwarding {} to user {} (backward compatibility)",
                broadcast_msg.event_type, user_id
            );
            broadcast_to_user(user_id, broadcast_msg).await;
        }
        None => {
            error!(
                "Progress message missing task_id or user_id: {:?}",
                broadcast_msg
            );
        }
    }
}

/// Create a message forwarder based on the configuration
pub async fn create_forwarder(
    config: ForwarderConfig,
) -> anyhow::Result<Box<dyn MessageForwarder>> {
    match config {
        ForwarderConfig::Redis { url, channel } => {
            let forwarder = RedisForwarder::new(&url, &channel).await?;
            Ok(Box::new(forwarder) as Box<dyn MessageForwarder>)
        }
        ForwarderConfig::Kafka {
            brokers,
            consumer_group,
            topic,
        } => {
            let forwarder =
                KafkaForwarder::new(&brokers, &topic, &format!("{}_forwarder", consumer_group))
                    .await?;
            Ok(Box::new(forwarder) as Box<dyn MessageForwarder>)
        }
        ForwarderConfig::RabbitMQ { url, queue } => {
            let forwarder = RabbitMQForwarder::new(&url, &queue).await?;
            Ok(Box::new(forwarder) as Box<dyn MessageForwarder>)
        }
    }
}
