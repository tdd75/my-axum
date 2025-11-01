use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio_cron_scheduler::JobScheduler;
use tracing::{error, info};

use crate::config::{setting::Setting, shutdown::wait_for_shutdown_signal};
use crate::core::db::connection::get_db;
use crate::pkg::messaging::{ConsumerConfig, create_consumer, create_producer};

use super::ConcreteTaskHandler;

/// Initialize and run the worker service
pub async fn run(setting: Setting) -> anyhow::Result<()> {
    info!("🚀 Starting worker service...");

    // Load consumer configuration from settings
    let consumer_config = setting.to_consumer_config()?;

    info!("Worker configuration loaded:");
    match &consumer_config {
        ConsumerConfig::Kafka {
            brokers,
            consumer_group,
            topics,
        } => {
            info!("  Broker: Kafka");
            info!("  Brokers: {}", brokers);
            info!("  Consumer group: {}", consumer_group);
            info!("  Topics: {:?}", topics);
        }
        ConsumerConfig::Redis { url, channels } => {
            info!("  Broker: Redis");
            info!("  URL: {}", url);
            info!("  Channels: {:?}", channels);
        }
        ConsumerConfig::RabbitMQ { url, queues } => {
            info!("  Broker: RabbitMQ");
            info!("  URL: {}", url);
            info!("  Queues: {:?}", queues);
        }
    }

    info!("  Worker pool size: {}", setting.messaging.worker_pool_size);
    info!("  Database: {}", setting.database_url);

    // Initialize task scheduler for periodic tasks
    let mut scheduler = JobScheduler::new().await?;
    info!("✓ Scheduler initialized");

    // Initialize database connection
    let db = get_db(&setting.database_url).await?;
    info!("✓ Database connection initialized");

    // Initialize SMTP client
    let smtp_client = setting
        .get_smtp_client()
        .map_err(|e| {
            error!("Failed to create SMTP client: {:?}", e);
            e
        })
        .ok();
    if smtp_client.is_some() {
        info!("✓ SMTP client initialized");
    } else {
        info!("⚠ SMTP client not configured (email tasks will fail)");
    }

    // Initialize message producer
    let producer_config = setting
        .to_producer_config()
        .ok_or_else(|| anyhow::anyhow!("Message broker is not configured for worker"))?;
    let producer = Arc::new(create_producer(producer_config).await?);
    info!("✓ Message producer initialized");

    // Initialize task handler
    let task_handler = Arc::new(ConcreteTaskHandler::new(
        db,
        producer.clone(),
        smtp_client,
        setting.redis_url.clone(),
    )?);
    info!("✓ Task handler initialized");

    // Initialize worker pool semaphore
    let semaphore = Arc::new(Semaphore::new(setting.messaging.worker_pool_size));
    info!(
        "✓ Worker pool initialized with {} workers",
        setting.messaging.worker_pool_size
    );

    // Initialize Consumer
    let mut consumer = create_consumer(
        consumer_config,
        task_handler.clone(),
        semaphore.clone(),
        producer.clone(),
    )
    .await?;
    info!("✓ {} Consumer initialized", consumer.broker_type());

    // Connect to consumer
    consumer.connect().await?;
    info!("✓ Connected to {} consumer", consumer.broker_type());

    // Start scheduler (for periodic tasks)
    scheduler.start().await?;
    info!("✓ Scheduler started");

    info!("🎯 Worker is ready and consuming messages...");
    info!("Press Ctrl+C to shutdown gracefully");

    // Start consuming messages
    tokio::select! {
        result = consumer.consume() => {
            match result {
                Ok(_) => info!("Consumer stopped"),
                Err(e) => {
                    tracing::error!("Consumer error: {:?}", e);
                    return Err(e);
                }
            }
        }
        _ = wait_for_shutdown_signal() => {
            info!("Shutting down worker...");
        }
    }

    // Cleanup
    consumer.close().await?;
    info!("✓ Consumer connection closed");
    scheduler.shutdown().await?;
    info!("✓ Scheduler stopped");
    info!("👋 Worker shutdown complete");

    Ok(())
}
