use serde::{Deserialize, Serialize};

use crate::messaging::MessageProducer;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct MockTask {
        name: String,
    }

    #[test]
    fn test_task_priority_as_score() {
        assert_eq!(TaskPriority::Low.as_score(), 0);
        assert_eq!(TaskPriority::Normal.as_score(), 1);
        assert_eq!(TaskPriority::High.as_score(), 2);
    }

    #[test]
    fn test_task_priority_default() {
        let priority: TaskPriority = Default::default();
        assert_eq!(priority, TaskPriority::Normal);
    }

    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Low < TaskPriority::Normal);
        assert!(TaskPriority::Normal < TaskPriority::High);
        assert!(TaskPriority::Low < TaskPriority::High);
    }

    #[test]
    fn test_task_event_new() {
        let task = MockTask {
            name: "test".to_string(),
        };
        let event = TaskEvent::new(task);

        assert!(!event.id.is_empty());
        assert_eq!(event.retry_count, 0);
        assert_eq!(event.max_retries, 3);
        assert_eq!(event.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_task_event_with_priority() {
        let task = MockTask {
            name: "high-priority".to_string(),
        };
        let event = TaskEvent::with_priority(task, TaskPriority::High);

        assert_eq!(event.priority, TaskPriority::High);
    }

    #[test]
    fn test_task_event_should_retry() {
        let task = MockTask {
            name: "retry".to_string(),
        };
        let mut event = TaskEvent::new(task);

        assert!(event.should_retry());

        event.retry_count = 3;
        assert!(!event.should_retry());
    }

    #[test]
    fn test_task_event_increment_retry() {
        let task = MockTask {
            name: "increment".to_string(),
        };
        let mut event = TaskEvent::new(task);

        assert_eq!(event.retry_count, 0);
        event.increment_retry();
        assert_eq!(event.retry_count, 1);
        event.increment_retry();
        assert_eq!(event.retry_count, 2);
    }

    #[test]
    fn test_task_event_serialization() {
        let task = MockTask {
            name: "serialize".to_string(),
        };
        let event = TaskEvent::new(task);

        let json = serde_json::to_string(&event).unwrap();
        let parsed: TaskEvent<MockTask> = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, event.id);
        assert_eq!(parsed.task.name, "serialize");
    }
}
