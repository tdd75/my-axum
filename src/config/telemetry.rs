use tracing::Subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry, fmt};

pub fn get_subscriber(log_dir: &str) -> impl Subscriber + Sync + Send {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Pretty console output
    let console_layer = fmt::layer()
        .pretty()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_writer(std::io::stdout);

    // File output with JSON format
    let file_writer = tracing_appender::rolling::daily(log_dir, "app.log");
    let file_layer = fmt::layer().json().with_writer(file_writer);

    Registry::default()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
}

#[cfg(test)]
mod tests {
    use tracing::Level;

    use super::get_subscriber;

    #[test]
    fn creates_subscriber() {
        let subscriber = get_subscriber("/tmp");
        assert!(std::mem::size_of_val(&subscriber) > 0);
    }

    #[test]
    fn subscriber_can_be_used_as_default_without_panicking() {
        let result = std::panic::catch_unwind(|| {
            let subscriber = tracing_subscriber::fmt()
                .with_max_level(Level::INFO)
                .with_test_writer()
                .finish();
            let _guard = tracing::subscriber::set_default(subscriber);
        });

        assert!(result.is_ok());
    }
}
