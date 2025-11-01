pub mod cron;
pub mod task;
pub mod worker;

use crate::pkg::messaging::MessageProducer;

// Re-export generic types from pkg::messaging::task
pub use crate::pkg::messaging::task::TaskPriority;

// Re-export application-specific task types and handler implementation
pub use task::{ConcreteTaskHandler, TaskType};

// Application-specific TaskEvent type
pub type TaskEvent = crate::pkg::messaging::task::TaskEvent<TaskType>;

/// Helper function to publish a task
pub async fn publish_task(
    producer: &dyn MessageProducer,
    task: TaskType,
    destination: Option<&str>,
) -> anyhow::Result<()> {
    let event = TaskEvent::new(task);
    publish_event(producer, &event, destination).await
}

/// Helper function to publish a task with priority
pub async fn publish_task_with_priority(
    producer: &dyn MessageProducer,
    task: TaskType,
    priority: TaskPriority,
    destination: Option<&str>,
) -> anyhow::Result<()> {
    let event = TaskEvent::with_priority(task, priority);
    publish_event(producer, &event, destination).await
}

/// Helper function to publish a task event
async fn publish_event(
    producer: &dyn MessageProducer,
    event: &TaskEvent,
    destination: Option<&str>,
) -> anyhow::Result<()> {
    let event_json = serde_json::to_string(event)
        .map_err(|e| anyhow::anyhow!("Failed to serialize event: {}", e))?;
    producer.publish_event_json(&event_json, destination).await
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use super::{TaskEvent, TaskPriority, TaskType, publish_task, publish_task_with_priority};
    use crate::pkg::messaging::MessageProducer;

    #[derive(Clone, Default)]
    struct MockProducer {
        published_events: Arc<Mutex<Vec<String>>>,
        fail_on_publish: Arc<Mutex<bool>>,
    }

    impl MockProducer {
        fn published_events(&self) -> Vec<String> {
            self.published_events.lock().unwrap().clone()
        }

        fn set_fail_on_publish(&self, fail: bool) {
            *self.fail_on_publish.lock().unwrap() = fail;
        }
    }

    #[async_trait]
    impl MessageProducer for MockProducer {
        async fn publish_event_json(
            &self,
            event_json: &str,
            _destination: Option<&str>,
        ) -> anyhow::Result<()> {
            if *self.fail_on_publish.lock().unwrap() {
                return Err(anyhow::anyhow!("Mock publish failure"));
            }

            self.published_events
                .lock()
                .unwrap()
                .push(event_json.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn publishes_task_with_default_priority() {
        let producer = MockProducer::default();

        publish_task(&producer, TaskType::CleanupExpiredToken, None)
            .await
            .unwrap();

        let event: TaskEvent = serde_json::from_str(&producer.published_events()[0]).unwrap();
        assert!(matches!(event.task, TaskType::CleanupExpiredToken));
        assert_eq!(event.priority, TaskPriority::Normal);
    }

    #[tokio::test]
    async fn publishes_task_with_custom_priority() {
        let producer = MockProducer::default();

        publish_task_with_priority(
            &producer,
            TaskType::ProcessUserRegistration { user_id: 42 },
            TaskPriority::High,
            Some("priority-topic"),
        )
        .await
        .unwrap();

        let event: TaskEvent = serde_json::from_str(&producer.published_events()[0]).unwrap();
        assert_eq!(event.priority, TaskPriority::High);
    }

    #[tokio::test]
    async fn surfaces_publish_errors() {
        let producer = MockProducer::default();
        producer.set_fail_on_publish(true);

        let error = publish_task(&producer, TaskType::CleanupExpiredToken, None)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("Mock publish failure"));
    }
}
