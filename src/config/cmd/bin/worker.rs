use dotenvy::dotenv;
use my_axum::config::{
    cmd::worker,
    cron::init_cron_job,
    setting::Setting,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();
    let setting = Setting::new();

    let subscriber = get_subscriber("logs/worker");
    init_subscriber(subscriber);

    init_cron_job(&setting).await.unwrap();
    worker::run(setting).await
}
