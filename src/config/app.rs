use std::{sync::Arc, time::Duration};

use crate::{
    config::{
        setting::{MessageBrokerType, Setting},
        shutdown::wait_for_shutdown_signal,
    },
    core::{
        api::route::{OPENAPI_JSON_PATH, SWAGGER_UI_PATH, get_route},
        db::connection::get_db,
        layer::{cors_layer::get_cors_layer, trace_layer::get_trace_layer},
    },
    pkg::{
        broadcast::forwarder::{ForwarderConfig, ShutdownSignal, create_forwarder},
        messaging::{MessageProducer, create_producer},
        url::UrlBuilder,
    },
};
use axum::Router;
use sea_orm::DatabaseConnection;
use tokio::{net::TcpListener, sync::watch, task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub setting: Setting,
    pub producer: Option<Arc<Box<dyn MessageProducer>>>,
    pub shutdown_token: CancellationToken,
}

pub struct App {
    listener: TcpListener,
    pub base_url: String,
    pub app_state: AppState,
}

fn print_startup_banner(server_url: &str) {
    let title = "My Axum Server Started";
    let entries = [
        ("Server URL", server_url.to_string()),
        ("Swagger UI", format!("{}{}", server_url, SWAGGER_UI_PATH)),
        (
            "OpenAPI JSON",
            format!("{}{}", server_url, OPENAPI_JSON_PATH),
        ),
    ];
    let label_width = entries
        .iter()
        .map(|(label, _)| label.chars().count())
        .max()
        .unwrap_or(0);
    let lines: Vec<String> = entries
        .iter()
        .map(|(label, value)| format!("{label:<label_width$} : {value}"))
        .collect();
    let width = std::iter::once(title.chars().count())
        .chain(lines.iter().map(|line| line.chars().count()))
        .max()
        .unwrap_or(0);
    let top_border = format!("╭{}╮", "─".repeat(width + 2));
    let divider = format!("├{}┤", "─".repeat(width + 2));
    let bottom_border = format!("╰{}╯", "─".repeat(width + 2));

    println!();
    println!("{}", top_border);
    println!("│ {:^width$} │", title, width = width);
    println!("{}", divider);
    for line in lines {
        println!("│ {:width$} │", line, width = width);
    }
    println!("{}", bottom_border);
    println!();
}

impl App {
    pub async fn new(setting: Setting) -> Result<Self, anyhow::Error> {
        let db = get_db(&setting.database_url).await?;
        Self::new_with_db(setting, db).await
    }

    pub async fn new_with_db(
        setting: Setting,
        db: DatabaseConnection,
    ) -> Result<Self, anyhow::Error> {
        // Bind using the configured host/port, then persist the actual socket address.
        let base_url = UrlBuilder::new(&setting.app_host)
            .port(setting.app_port)
            .build();
        let listener = TcpListener::bind(base_url.as_str()).await?;
        let local_addr = listener.local_addr()?;
        let mut setting = setting;
        setting.app_host = local_addr.ip().to_string();
        setting.app_port = local_addr.port();

        // Initialize message producer (optional)
        let producer = if let Some(producer_config) = setting.to_producer_config() {
            let p = create_producer(producer_config).await?;
            tracing::info!("Message producer initialized successfully");
            Some(Arc::new(p))
        } else {
            tracing::info!("Message producer disabled (no broker configured)");
            None
        };

        Ok(Self {
            listener,
            base_url: local_addr.to_string(),
            app_state: AppState {
                db,
                setting,
                producer,
                shutdown_token: CancellationToken::new(),
            },
        })
    }

    fn spawn_message_forwarder(
        setting: Setting,
        shutdown: ShutdownSignal,
    ) -> Option<JoinHandle<()>> {
        let broker_type = setting.messaging.message_broker.clone()?;

        Some(tokio::spawn(async move {
            let forwarder_config = match broker_type {
                MessageBrokerType::Kafka => ForwarderConfig::kafka(
                    setting.messaging.kafka_brokers.clone(),
                    "broadcasts".to_string(),
                    setting.messaging.kafka_consumer_group.clone(),
                ),
                MessageBrokerType::Redis => ForwarderConfig::redis(
                    setting.messaging.redis_url.clone(),
                    "broadcasts".to_string(),
                ),
                MessageBrokerType::RabbitMQ => ForwarderConfig::rabbitmq(
                    setting.messaging.rabbitmq_url.clone(),
                    "broadcasts".to_string(),
                ),
            };

            match create_forwarder(forwarder_config).await {
                Ok(forwarder) => {
                    tracing::info!("✓ Message forwarder started for progress updates");
                    if let Err(e) = forwarder.start_forwarding(shutdown).await {
                        tracing::error!("Message forwarder error: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to start message forwarder: {}", e);
                }
            }
        }))
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        let Self {
            listener,
            base_url,
            app_state,
        } = self;

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let forwarder_handle =
            Self::spawn_message_forwarder(app_state.setting.clone(), shutdown_rx);

        let server_url = format!("http://{}", base_url);
        print_startup_banner(&server_url);

        let db = app_state.db.clone();
        let shutdown_token = app_state.shutdown_token.clone();
        let app = Router::new()
            .merge(get_route(app_state.clone()))
            .with_state(app_state)
            .layer(get_cors_layer())
            .layer(get_trace_layer());

        let shutdown_server = {
            let shutdown_tx = shutdown_tx.clone();
            async move {
                wait_for_shutdown_signal().await;
                shutdown_token.cancel();
                let _ = shutdown_tx.send(true);
            }
        };

        let result = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_server)
            .await;

        let _ = shutdown_tx.send(true);

        if let Some(mut handle) = forwarder_handle {
            tokio::select! {
                join_result = &mut handle => {
                    if let Err(error) = join_result {
                        tracing::warn!("Message forwarder task ended unexpectedly: {}", error);
                    }
                }
                _ = sleep(Duration::from_secs(5)) => {
                    tracing::warn!("Message forwarder did not stop in time; aborting task");
                    handle.abort();
                    if let Err(error) = handle.await
                        && !error.is_cancelled()
                    {
                        tracing::warn!("Message forwarder task ended unexpectedly: {}", error);
                    }
                }
            }
        }

        if let Err(error) = db.close().await {
            tracing::warn!("Failed to close database connection cleanly: {}", error);
        }

        result
    }
}
