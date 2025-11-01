use async_trait::async_trait;
use futures::StreamExt;
use redis::{Client, aio::PubSub};
use tracing::{error, info};

use super::{MessageForwarder, forward_message_to_websocket};

/// Redis subscriber that forwards progress updates to WebSockets
pub struct RedisForwarder {
    client: Client,
    channel: String,
}

impl RedisForwarder {
    /// Create a new Redis forwarder
    pub async fn new(redis_url: &str, channel: &str) -> anyhow::Result<Self> {
        let client = Client::open(redis_url)?;

        // Test connection
        let _conn = client.get_multiplexed_async_connection().await?;
        info!("✓ Redis forwarder initialized for channel: {}", channel);

        Ok(Self {
            client,
            channel: channel.to_string(),
        })
    }
}

#[async_trait]
impl MessageForwarder for RedisForwarder {
    async fn start_forwarding(self: Box<Self>) -> anyhow::Result<()> {
        let this = *self;
        let mut pubsub: PubSub = this.client.get_async_pubsub().await?;

        pubsub.subscribe(&this.channel).await?;
        info!("✓ Redis forwarder subscribed to channel: {}", this.channel);

        let mut stream = pubsub.on_message();

        loop {
            match stream.next().await {
                Some(msg) => {
                    let payload: String = match msg.get_payload() {
                        Ok(p) => p,
                        Err(e) => {
                            error!("Failed to get payload from Redis message: {}", e);
                            continue;
                        }
                    };

                    forward_message_to_websocket(&payload).await;
                }
                None => {
                    error!("Redis pubsub stream ended");
                    break;
                }
            }
        }

        Ok(())
    }
}
