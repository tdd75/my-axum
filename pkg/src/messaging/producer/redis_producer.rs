use anyhow::{Context, Result};
use async_trait::async_trait;
use redis::{AsyncCommands, Client, aio::MultiplexedConnection};

use super::{MessageProducer, event_id_from_json};

/// Redis producer implementation (Pub/Sub)
pub struct RedisProducer {
    connection: MultiplexedConnection,
    default_channel: String,
}

impl RedisProducer {
    pub async fn new(url: &str, default_channel: &str) -> Result<Self> {
        let client = Client::open(url).context("Failed to create Redis client")?;

        let connection = client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to connect to Redis")?;

        Ok(Self {
            connection,
            default_channel: default_channel.to_string(),
        })
    }
}

#[async_trait]
impl MessageProducer for RedisProducer {
    async fn publish_event_json(
        &self,
        event_json: &str,
        destination: Option<&str>,
    ) -> anyhow::Result<()> {
        let channel = destination.unwrap_or(&self.default_channel);

        let event_id = event_id_from_json(event_json);
        let mut conn = self.connection.clone();

        let subscriber_count: i32 = conn
            .publish(channel, event_json)
            .await
            .context("Failed to publish message to Redis")?;

        if subscriber_count == 0 {
            tracing::warn!(
                "⚠️  Published task event {} to Redis channel {} but no subscribers are listening!",
                event_id,
                channel
            );
        } else {
            tracing::info!(
                "✓ Published task event {} to Redis channel {} ({} subscribers)",
                event_id,
                channel,
                subscriber_count
            );
        }

        Ok(())
    }
}
