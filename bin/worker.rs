use dotenvy::dotenv;
use my_axum::config::{
    cmd::worker,
    cron::init_cron_job,
    setting::Setting,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load settings from .env file
    let _ = dotenv();
    let setting = Setting::new();

    // Initialize telemetry
    let subscriber = get_subscriber("logs/worker");
    init_subscriber(subscriber);

    // Init cron tasks
    init_cron_job(&setting).await.unwrap();

    // Run worker
    worker::run(setting).await?;

    Ok(())
}
