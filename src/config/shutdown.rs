use tracing::{error, info};

pub async fn wait_for_shutdown_signal() {
    match shutdown_signal().await {
        Ok(signal) => info!(signal, "Shutdown signal received"),
        Err(error) => error!("Failed to listen for shutdown signal: {error}"),
    }
}

async fn shutdown_signal() -> std::io::Result<&'static str> {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await?;
        Ok::<_, std::io::Error>("SIGINT")
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};

        let mut signal = signal(SignalKind::terminate())?;
        signal.recv().await;
        Ok::<_, std::io::Error>("SIGTERM")
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<std::io::Result<&'static str>>();

    tokio::select! {
        signal = ctrl_c => signal,
        signal = terminate => signal,
    }
}
