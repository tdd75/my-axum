use dotenvy::dotenv;
use my_axum::config::{cmd::seed, setting::Setting};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    let _ = dotenv();

    // Get settings
    let setting = Setting::new();

    // Run seeding
    seed::run(&setting).await?;

    Ok(())
}
