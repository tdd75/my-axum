use serde::{Deserialize, Serialize};

use crate::pkg::messaging::MessageProducer;

/// Priority level for task execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TaskPriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
}

impl TaskPriority {
    /// Convert priority to numeric value for ordering
    pub fn as_score(&self) -> i64 {
        match self {
            TaskPriority::Low => 0,
            TaskPriority::Normal => 1,
            TaskPriority::High => 2,
        }
    }
}

/// Generic event wrapper with metadata
/// The task payload is generic and can be any serializable type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent<T>
where
    T: Clone + Send + Sync,
{
    pub id: String,
    pub task: T,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub priority: TaskPriority,
}

impl<T> TaskEvent<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(task: T) -> Self {
        Self::with_priority(task, TaskPriority::Normal)
    }

    pub fn with_priority(task: T, priority: TaskPriority) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task,
            created_at: chrono::Utc::now(),
            retry_count: 0,
            max_retries: 3,
            priority,
        }
    }

    pub fn should_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Helper method to publish this event using a producer
    pub async fn publish_with_producer(
        &self,
        producer: &dyn MessageProducer,
        destination: Option<&str>,
    ) -> anyhow::Result<()>
    where
        T: serde::Serialize,
    {
        let event_json = serde_json::to_string(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize event: {}", e))?;
        producer.publish_event_json(&event_json, destination).await
    }
}
