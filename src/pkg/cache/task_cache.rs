use anyhow::{Context, Result};
use redis::{Client, aio::MultiplexedConnection};
use serde_json::Value;

const TASK_STATUS_KEY_PREFIX: &str = "task:status:";
const TASK_STATUS_TTL: i64 = 3600; // 1 hour TTL

/// Helper struct for caching task status in Redis
pub struct TaskStatusCache {
    connection: MultiplexedConnection,
}

impl TaskStatusCache {
    /// Create a new TaskStatusCache
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url).context("Failed to create Redis client")?;
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to connect to Redis")?;

        Ok(Self { connection })
    }

    /// Cache task status in Redis
    pub async fn cache_status(&mut self, task_id: &str, status_data: &Value) -> Result<()> {
        let key = format!("{}{}", TASK_STATUS_KEY_PREFIX, task_id);
        let json_str =
            serde_json::to_string(status_data).context("Failed to serialize task status")?;

        let _: () = redis::cmd("SETEX")
            .arg(&key)
            .arg(TASK_STATUS_TTL)
            .arg(&json_str)
            .query_async(&mut self.connection)
            .await
            .context("Failed to cache task status in Redis")?;

        tracing::debug!("Cached task status for task_id: {}", task_id);
        Ok(())
    }

    /// Get cached task status from Redis
    pub async fn get_status(&mut self, task_id: &str) -> Result<Option<Value>> {
        use redis::AsyncCommands;

        let key = format!("{}{}", TASK_STATUS_KEY_PREFIX, task_id);

        let json_str: Option<String> = self
            .connection
            .get(&key)
            .await
            .context("Failed to get task status from Redis")?;

        match json_str {
            Some(json) => {
                let status_data =
                    serde_json::from_str(&json).context("Failed to deserialize task status")?;
                tracing::debug!("Retrieved cached task status for task_id: {}", task_id);
                Ok(Some(status_data))
            }
            None => {
                tracing::debug!("No cached task status found for task_id: {}", task_id);
                Ok(None)
            }
        }
    }
}

/// Helper function to cache task status
pub async fn cache_task_status(redis_url: &str, task_id: &str, status_data: &Value) -> Result<()> {
    let mut cache = TaskStatusCache::new(redis_url).await?;
    cache.cache_status(task_id, status_data).await
}

/// Helper function to get cached task status
pub async fn get_cached_task_status(redis_url: &str, task_id: &str) -> Result<Option<Value>> {
    let mut cache = TaskStatusCache::new(redis_url).await?;
    cache.get_status(task_id).await
}
