use dotenvy::dotenv;
use my_axum::config::{
    app::App,
    setting::Setting,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() {
    // Load settings from .env file
    let _ = dotenv();
    let setting = Setting::new();

    // Initialize telemetry
    let subscriber = get_subscriber("logs/axum");
    init_subscriber(subscriber);

    // Run the application
    let app = App::new(setting).await.unwrap();
    app.run_until_stopped().await.unwrap();
}
