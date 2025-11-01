use async_trait::async_trait;
use futures::StreamExt;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable};
use tracing::{error, info, warn};

use super::{MessageForwarder, ShutdownSignal, forward_message_to_websocket};

/// RabbitMQ consumer that forwards progress updates to WebSockets
pub struct RabbitMQForwarder {
    connection: Connection,
    queue: String,
}

impl RabbitMQForwarder {
    /// Create a new RabbitMQ forwarder
    pub async fn new(url: &str, queue: &str) -> anyhow::Result<Self> {
        let connection = Connection::connect(url, ConnectionProperties::default()).await?;
        info!("✓ RabbitMQ forwarder initialized for queue: {}", queue);

        Ok(Self {
            connection,
            queue: queue.to_string(),
        })
    }
}

#[async_trait]
impl MessageForwarder for RabbitMQForwarder {
    async fn start_forwarding(self: Box<Self>, mut shutdown: ShutdownSignal) -> anyhow::Result<()> {
        let this = *self;
        let channel = this.connection.create_channel().await?;

        // Declare queue if it doesn't exist
        channel
            .queue_declare(
                this.queue.as_str().into(),
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        let mut consumer = channel
            .basic_consume(
                this.queue.as_str().into(),
                "websocket_forwarder".into(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!("✓ RabbitMQ forwarder subscribed to queue: {}", this.queue);

        loop {
            tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        info!("Shutting down RabbitMQ forwarder");
                        break;
                    }
                }
                message = consumer.next() => {
                    match message {
                        Some(Ok(delivery)) => {
                            if let Ok(payload) = std::str::from_utf8(&delivery.data) {
                                forward_message_to_websocket(payload).await;
                            } else {
                                error!("Failed to decode RabbitMQ message payload");
                            }

                            // Acknowledge the message
                            if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                                error!("Failed to ack RabbitMQ message: {}", e);
                            }
                        }
                        Some(Err(e)) => {
                            error!("RabbitMQ consumer error: {}", e);
                        }
                        None => {
                            error!("RabbitMQ consumer stream ended");
                            break;
                        }
                    }
                }
            }
        }

        if let Err(error) = channel.close(0, "Normal shutdown".into()).await {
            warn!("Failed to close RabbitMQ forwarder channel: {}", error);
        }
        if let Err(error) = this.connection.close(0, "Normal shutdown".into()).await {
            warn!("Failed to close RabbitMQ forwarder connection: {}", error);
        }

        Ok(())
    }
}
