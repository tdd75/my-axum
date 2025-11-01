use anyhow::Context;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    client::DefaultClientContext,
    config::ClientConfig,
};
use tracing::{info, warn};

/// Ensure Kafka topics exist, create them if they don't
pub async fn ensure_topics_exist(brokers: &str, topics: &[&str]) -> anyhow::Result<()> {
    info!("Checking if Kafka topics exist: {:?}", topics);

    let admin_client: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()
        .context("Failed to create Kafka admin client")?;

    // Create topic specifications
    let new_topics: Vec<NewTopic> = topics
        .iter()
        .map(|topic| {
            NewTopic::new(
                topic,
                1,                          // num_partitions
                TopicReplication::Fixed(1), // replication_factor
            )
        })
        .collect();

    // Try to create topics
    let opts = AdminOptions::new();
    let results = admin_client
        .create_topics(&new_topics, &opts)
        .await
        .context("Failed to create topics")?;

    // Check results
    for (topic, result) in topics.iter().zip(results.iter()) {
        match result {
            Ok(_) => info!("✓ Topic '{}' created successfully", topic),
            Err(e) => {
                // Topic already exists is not an error
                if format!("{:?}", e).contains("TopicAlreadyExists") {
                    info!("✓ Topic '{}' already exists", topic);
                } else {
                    warn!("Warning creating topic '{}': {:?}", topic, e);
                }
            }
        }
    }

    Ok(())
}
