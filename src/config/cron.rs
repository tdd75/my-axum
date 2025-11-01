use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::config::setting::{MessageType, Setting};
use crate::core::db::connection::get_db;
use crate::core::task::{TaskType, publish_task};
use crate::pkg::messaging::{MessageProducer, create_producer};

/// Creates a cron job that publishes a task to the message broker
fn create_job_with_task<C, F, Fut>(
    schedule: &str,
    context: C,
    producer: Arc<Box<dyn MessageProducer>>,
    task_fn: F,
) -> Result<Job, anyhow::Error>
where
    C: Send + Sync + Clone + 'static,
    F: Fn(C, Arc<Box<dyn MessageProducer>>) -> Fut + Send + Sync + 'static + Clone,
    Fut: std::future::Future<Output = Result<(), anyhow::Error>> + Send + 'static,
{
    Ok(Job::new_async(schedule, move |uuid, mut l| {
        let producer = producer.clone();
        let context = context.clone();
        let task_fn = task_fn.clone();
        Box::pin(async move {
            let next_tick = l.next_tick_for_job(uuid).await;
            match next_tick {
                Ok(Some(ts)) => {
                    tracing::info!("Executing scheduled job at {}", ts.to_rfc3339());

                    if let Err(e) = task_fn(context, producer).await {
                        tracing::error!("Failed to execute job: {:?}", e);
                    } else {
                        tracing::info!("✓ Job executed successfully");
                    }
                }
                _ => tracing::warn!("Could not get next tick for job"),
            }
        })
    })?)
}

pub async fn init_cron_job(setting: &Setting) -> Result<(), anyhow::Error> {
    tracing::info!("Initializing cron jobs with message broker...");
    let sched = JobScheduler::new().await?;

    // Initialize database connection
    let db = get_db(&setting.database_url).await?;
    tracing::info!("✓ Database connected for cron jobs");

    // Create message producer for publishing jobs
    let producer_config = setting
        .to_producer_config()
        .ok_or_else(|| anyhow::anyhow!("Message broker is not configured for cron jobs"))?;
    let producer = Arc::new(create_producer(producer_config).await?);
    tracing::info!("✓ Message producer initialized for cron jobs");

    // Clean expired tokens every hour
    let cleanup_job = create_job_with_task(
        "0 0 * * * *", // Every hour at minute 0
        db.clone(),
        producer.clone(),
        |_db, producer| async move {
            publish_task(
                producer.as_ref().as_ref(),
                TaskType::CleanupExpiredToken,
                Some(MessageType::Tasks.as_ref()),
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to publish cleanup job: {:?}", e))
        },
    )?;
    sched.add(cleanup_job).await?;

    // Start the scheduler
    sched.start().await?;
    tracing::info!("✓ Cron scheduler started");

    Ok(())
}
