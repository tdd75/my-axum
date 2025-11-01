use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};
use tracing::{error, info, warn};

use crate::messaging::{MessageProducer, TaskEvent, TaskHandler};

#[derive(Clone)]
pub(super) struct PriorityTask<T>
where
    T: Clone + Send + Sync,
{
    event: TaskEvent<T>,
}

impl<T> PartialEq for PriorityTask<T>
where
    T: Clone + Send + Sync,
{
    fn eq(&self, other: &Self) -> bool {
        self.event.priority == other.event.priority
            && self.event.created_at == other.event.created_at
    }
}

impl<T> Eq for PriorityTask<T> where T: Clone + Send + Sync {}

impl<T> PartialOrd for PriorityTask<T>
where
    T: Clone + Send + Sync,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for PriorityTask<T>
where
    T: Clone + Send + Sync,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.event.priority.cmp(&other.event.priority) {
            Ordering::Equal => other.event.created_at.cmp(&self.event.created_at),
            other_order => other_order,
        }
    }
}

pub(super) type SharedPriorityQueue<T> = Arc<Mutex<BinaryHeap<PriorityTask<T>>>>;

pub(super) fn new_priority_queue<T>() -> SharedPriorityQueue<T>
where
    T: Clone + Send + Sync,
{
    Arc::new(Mutex::new(BinaryHeap::new()))
}

pub(super) async fn enqueue_task<T>(priority_queue: &SharedPriorityQueue<T>, event: TaskEvent<T>)
where
    T: Clone + Send + Sync,
{
    let mut queue = priority_queue.lock().await;
    queue.push(PriorityTask { event });
}

pub(super) fn spawn_priority_processor<T>(
    priority_queue: SharedPriorityQueue<T>,
    task_handler: Arc<dyn TaskHandler<T>>,
    semaphore: Arc<Semaphore>,
    producer: Arc<Box<dyn MessageProducer>>,
) where
    T: Clone + Send + Sync + Serialize + 'static,
{
    tokio::spawn(async move {
        run_priority_processor(priority_queue, task_handler, semaphore, producer).await;
    });
}

async fn run_priority_processor<T>(
    priority_queue: SharedPriorityQueue<T>,
    task_handler: Arc<dyn TaskHandler<T>>,
    semaphore: Arc<Semaphore>,
    producer: Arc<Box<dyn MessageProducer>>,
) where
    T: Clone + Send + Sync + Serialize + 'static,
{
    loop {
        let task = {
            let mut queue = priority_queue.lock().await;
            queue.pop()
        };

        match task {
            Some(priority_task) => {
                let event = priority_task.event;
                info!(
                    "Processing task {} with priority {:?}",
                    event.id, event.priority
                );

                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Failed to acquire semaphore: {:?}", e);
                        continue;
                    }
                };
                let handler = task_handler.clone();
                let producer_clone = producer.clone();

                tokio::spawn(async move {
                    let _permit = permit;

                    match handler.handle_task(&event).await {
                        Ok(_) => info!("Task {} completed successfully", event.id),
                        Err(error) => handle_task_failure(event, producer_clone, error).await,
                    }
                });
            }
            None => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn handle_task_failure<T>(
    event: TaskEvent<T>,
    producer: Arc<Box<dyn MessageProducer>>,
    error: anyhow::Error,
) where
    T: Clone + Send + Sync + Serialize + 'static,
{
    error!("Task {} failed: {:?}", event.id, error);

    if !event.should_retry() {
        error!(
            "Task {} exceeded max retries ({})",
            event.id, event.max_retries
        );
        return;
    }

    warn!(
        "Task {} will be retried (attempt {}/{})",
        event.id,
        event.retry_count + 1,
        event.max_retries
    );

    let mut retry_event = event.clone();
    retry_event.increment_retry();

    let delay_secs = 2_u64.pow(retry_event.retry_count);
    info!(
        "Task {} will retry in {} seconds",
        retry_event.id, delay_secs
    );

    tokio::spawn(async move {
        retry_task_after_delay(retry_event, producer, delay_secs).await;
    });
}

async fn retry_task_after_delay<T>(
    retry_event: TaskEvent<T>,
    producer: Arc<Box<dyn MessageProducer>>,
    delay_secs: u64,
) where
    T: Clone + Send + Sync + Serialize + 'static,
{
    tokio::time::sleep(Duration::from_secs(delay_secs)).await;

    match retry_event
        .publish_with_producer(producer.as_ref().as_ref(), None)
        .await
    {
        Ok(_) => info!("Task {} republished for retry", retry_event.id),
        Err(error) => error!("Failed to republish task {}: {:?}", retry_event.id, error),
    }
}
