pub mod task_handler;
pub mod task_type;

use crate::pkg::messaging::MessageProducer;

// Re-export generic types from pkg::messaging::task
pub use crate::pkg::messaging::task::TaskPriority;

// Application-specific TaskEvent type
pub type TaskEvent = crate::pkg::messaging::task::TaskEvent<TaskType>;

// Re-export application-specific task types
pub use task_type::TaskType;

// Re-export the concrete handler implementation
pub use task_handler::ConcreteTaskHandler;

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
