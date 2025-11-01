use std::sync::Arc;

use crate::{
    config::setting::{MessageBrokerType, Setting},
    core::{
        api::route::get_route,
        db::connection::get_db,
        layer::{cors_layer::get_cors_layer, trace_layer::get_trace_layer},
    },
    pkg::{
        broadcast::forwarder::{ForwarderConfig, create_forwarder},
        messaging::{MessageProducer, create_producer},
        url::UrlBuilder,
    },
};
use axum::Router;
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub setting: Setting,
    pub producer: Option<Arc<Box<dyn MessageProducer>>>,
}

pub struct App {
    listener: TcpListener,
    pub base_url: String,
    pub app_state: AppState,
}

impl App {
    pub async fn new(setting: Setting) -> Result<Self, anyhow::Error> {
        // Bind to a random available port
        let base_url = UrlBuilder::new(&setting.app_host)
            .port(setting.app_port)
            .build();
        let listener = TcpListener::bind(base_url.as_str()).await?;
        let real_port = listener.local_addr()?.port();

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
            base_url: UrlBuilder::new(&setting.app_host).port(real_port).build(),
            app_state: AppState {
                db: get_db(&setting.database_url).await?,
                setting: setting.clone(),
                producer,
            },
        })
    }

    fn spawn_message_forwarder(setting: Setting) {
        tokio::spawn(async move {
            let broker_type = match setting.messaging.message_broker {
                Some(bt) => bt,
                None => return,
            };

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
                    if let Err(e) = forwarder.start_forwarding().await {
                        tracing::error!("Message forwarder error: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to start message forwarder: {}", e);
                }
            }
        });
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        // Start message forwarder in background if message broker is configured
        Self::spawn_message_forwarder(self.app_state.setting.clone());

        let app = Router::new()
            .merge(get_route(self.app_state.clone()))
            .with_state(self.app_state)
            .layer(get_cors_layer())
            .layer(get_trace_layer());

        axum::serve(self.listener, app).into_future().await
    }
}
