use async_trait::async_trait;

use super::TaskEvent;

/// Trait for handling task events
/// T is the application-specific task type
#[async_trait]
pub trait TaskHandler<T>: Send + Sync
where
    T: Clone + Send + Sync,
{
    /// Process a task event
    async fn handle_task(&self, event: &TaskEvent<T>) -> anyhow::Result<()>;
}
