use serde::Deserialize;
use std::{env::var, sync::LazyLock};
use strum::{AsRefStr, VariantNames};

use crate::pkg::{
    messaging::{ConsumerConfig, ProducerConfig},
    smtp::{SmtpClient, SmtpConfig},
};

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageBrokerType {
    Kafka,
    Redis,
    RabbitMQ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, VariantNames)]
#[strum(serialize_all = "lowercase")]
pub enum MessageType {
    Events,
    Tasks,
    Emails,
}

impl MessageType {
    pub fn all_as_string() -> String {
        Self::VARIANTS.join(",")
    }

    pub fn default_str() -> &'static str {
        MessageType::Tasks.as_ref()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Setting {
    pub app_host: String,
    pub app_port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub jwt_access_token_expires: i64,
    pub jwt_refresh_token_expires: i64,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_tls: bool,
    pub app_url: String,
    pub smtp_user: Option<String>,
    pub smtp_password: Option<String>,
    pub allowed_origins: Vec<String>,
    pub page_size_limit: Option<u64>,
    pub messaging: MessagingSetting,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessagingSetting {
    // Message broker type (kafka, redis, rabbitmq, or None to disable)
    pub message_broker: Option<MessageBrokerType>,
    // Worker settings
    pub worker_pool_size: usize,
    // Kafka settings
    pub kafka_brokers: String,
    pub kafka_consumer_group: String,
    pub kafka_topics: String,
    pub kafka_default_topic: String,
    // Redis settings
    pub redis_url: String,
    pub redis_channels: String,
    pub redis_default_channel: String,
    // RabbitMQ settings
    pub rabbitmq_url: String,
    pub rabbitmq_exchange: String,
    pub rabbitmq_queue: String,
    pub rabbitmq_queues: String,
    pub rabbitmq_default_queue: String,
}

// Global cached instance - initialized once on first access
static SETTING: LazyLock<Setting> = LazyLock::new(Setting::load_from_env);

impl Default for Setting {
    fn default() -> Self {
        Self::new()
    }
}

impl Setting {
    /// Get a cached reference to the global Setting instance
    pub fn new() -> Self {
        SETTING.clone()
    }

    /// Load settings from environment variables (internal use)
    fn load_from_env() -> Self {
        Self {
            app_host: var("APP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            app_port: var("APP_PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .unwrap_or(8000),
            database_url: var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:password@localhost:5432/my_axum".to_string()
            }),
            redis_url: var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: var("JWT_SECRET").unwrap_or_else(|_| "very-secured-secret".to_string()),
            jwt_access_token_expires: var("JWT_ACCESS_TOKEN_EXPIRES")
                .unwrap_or_else(|_| "1800".to_string()) // 30 minutes
                .parse()
                .unwrap_or(1800),
            jwt_refresh_token_expires: var("JWT_REFRESH_TOKEN_EXPIRES")
                .unwrap_or_else(|_| "604800".to_string()) // 7 days
                .parse()
                .unwrap_or(86400),
            app_url: var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
            smtp_host: var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_port: var("SMTP_PORT")
                .unwrap_or_else(|_| "465".to_string())
                .parse()
                .unwrap_or(465),
            smtp_tls: var("SMTP_TLS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            smtp_user: var("SMTP_USER").ok(),
            smtp_password: var("SMTP_PASSWORD").ok(),
            allowed_origins: var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            page_size_limit: var("PAGE_SIZE_LIMIT")
                .ok()
                .and_then(|value| value.parse().ok())
                .filter(|limit| *limit > 0),
            // Messaging settings
            messaging: MessagingSetting {
                message_broker: var("MESSAGE_BROKER").ok().and_then(|s| {
                    match s.to_lowercase().as_str() {
                        "kafka" => Some(MessageBrokerType::Kafka),
                        "redis" => Some(MessageBrokerType::Redis),
                        "rabbitmq" | "amqp" => Some(MessageBrokerType::RabbitMQ),
                        _ => None,
                    }
                }),
                worker_pool_size: var("WORKER_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                // Kafka
                kafka_brokers: var("KAFKA_BROKERS")
                    .unwrap_or_else(|_| "localhost:19092".to_string()),
                kafka_consumer_group: var("KAFKA_CONSUMER_GROUP")
                    .unwrap_or_else(|_| "my-axum-workers".to_string()),
                kafka_topics: var("KAFKA_TOPICS").unwrap_or_else(|_| MessageType::all_as_string()),
                kafka_default_topic: var("KAFKA_DEFAULT_TOPIC")
                    .unwrap_or_else(|_| MessageType::default_str().to_string()),
                // Redis
                redis_url: var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                redis_channels: var("REDIS_CHANNELS")
                    .unwrap_or_else(|_| MessageType::all_as_string()),
                redis_default_channel: var("REDIS_DEFAULT_CHANNEL")
                    .unwrap_or_else(|_| MessageType::default_str().to_string()),
                // RabbitMQ
                rabbitmq_url: var("RABBITMQ_URL")
                    .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".to_string()),
                rabbitmq_exchange: var("RABBITMQ_EXCHANGE")
                    .unwrap_or_else(|_| "my_axum_exchange".to_string()),
                rabbitmq_queue: var("RABBITMQ_QUEUE")
                    .unwrap_or_else(|_| MessageType::default_str().to_string()),
                rabbitmq_queues: var("RABBITMQ_QUEUES")
                    .unwrap_or_else(|_| MessageType::all_as_string()),
                rabbitmq_default_queue: var("RABBITMQ_DEFAULT_QUEUE")
                    .unwrap_or_else(|_| MessageType::default_str().to_string()),
            },
        }
    }

    pub fn get_smtp_client(&self) -> Result<SmtpClient, anyhow::Error> {
        if self.smtp_user.is_none() || self.smtp_password.is_none() {
            return Err(anyhow::anyhow!(
                "SMTP_USER or SMTP_PASSWORD is not set in environment"
            ));
        }

        let smtp_config = SmtpConfig::new(
            self.smtp_host.clone(),
            self.smtp_port,
            self.smtp_user.clone().unwrap(),
            self.smtp_password.clone().unwrap(),
            self.smtp_tls,
        );

        let smtp_client = SmtpClient::new(smtp_config)
            .map_err(|e| anyhow::anyhow!("Failed to create SMTP client: {}", e))?;

        Ok(smtp_client)
    }

    /// Create ConsumerConfig from settings
    pub fn to_consumer_config(&self) -> anyhow::Result<ConsumerConfig> {
        self.messaging.to_consumer_config()
    }

    /// Create ProducerConfig from settings
    pub fn to_producer_config(&self) -> Option<ProducerConfig> {
        self.messaging.to_producer_config()
    }
}

impl MessagingSetting {
    /// Create ConsumerConfig from messaging settings
    pub fn to_consumer_config(&self) -> anyhow::Result<ConsumerConfig> {
        let broker_type = self
            .message_broker
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Message broker is not configured"))?;

        match broker_type {
            MessageBrokerType::Kafka => {
                let topics: Vec<String> = self
                    .kafka_topics
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                Ok(ConsumerConfig::kafka(
                    self.kafka_brokers.clone(),
                    self.kafka_consumer_group.clone(),
                    topics,
                ))
            }
            MessageBrokerType::Redis => {
                let channels: Vec<String> = self
                    .redis_channels
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                Ok(ConsumerConfig::redis(self.redis_url.clone(), channels))
            }
            MessageBrokerType::RabbitMQ => {
                let queues: Vec<String> = self
                    .rabbitmq_queues
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                Ok(ConsumerConfig::rabbitmq(self.rabbitmq_url.clone(), queues))
            }
        }
    }

    /// Create ProducerConfig from settings
    pub fn to_producer_config(&self) -> Option<ProducerConfig> {
        self.message_broker
            .as_ref()
            .map(|broker_type| match broker_type {
                MessageBrokerType::Kafka => ProducerConfig::kafka(
                    self.kafka_brokers.clone(),
                    self.kafka_default_topic.clone(),
                ),
                MessageBrokerType::Redis => ProducerConfig::redis(
                    self.redis_url.clone(),
                    self.redis_default_channel.clone(),
                ),
                MessageBrokerType::RabbitMQ => ProducerConfig::rabbitmq(
                    self.rabbitmq_url.clone(),
                    self.rabbitmq_default_queue.clone(),
                ),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{MessageBrokerType, MessageType, MessagingSetting, Setting};

    fn sample_messaging_setting(message_broker: Option<MessageBrokerType>) -> MessagingSetting {
        MessagingSetting {
            message_broker,
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "topic1,topic2".to_string(),
            kafka_default_topic: "topic1".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "channel1,channel2".to_string(),
            redis_default_channel: "channel1".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "exchange".to_string(),
            rabbitmq_queue: "queue".to_string(),
            rabbitmq_queues: "queue1,queue2".to_string(),
            rabbitmq_default_queue: "queue1".to_string(),
        }
    }

    #[test]
    fn exposes_default_setting_values() {
        let setting = Setting::new();
        assert!(!setting.app_host.is_empty());
        assert!(setting.app_port > 0);
        assert!(!setting.database_url.is_empty());
        assert!(!setting.jwt_secret.is_empty());
    }

    #[test]
    fn leaves_page_size_unlimited_when_not_configured() {
        let setting = Setting::new();

        assert!(setting.page_size_limit.is_none() || setting.page_size_limit.unwrap() > 0);
    }

    #[test]
    fn clones_setting() {
        let setting = Setting::new();
        let cloned = setting.clone();

        assert_eq!(setting.app_host, cloned.app_host);
        assert_eq!(setting.database_url, cloned.database_url);
    }

    #[test]
    fn exposes_message_type_helpers() {
        assert_eq!(MessageType::default_str(), "tasks");
        assert_eq!(MessageType::all_as_string(), "events,tasks,emails");
    }

    #[test]
    fn builds_consumer_config_for_each_broker() {
        assert!(
            sample_messaging_setting(Some(MessageBrokerType::Kafka))
                .to_consumer_config()
                .is_ok()
        );
        assert!(
            sample_messaging_setting(Some(MessageBrokerType::Redis))
                .to_consumer_config()
                .is_ok()
        );
        assert!(
            sample_messaging_setting(Some(MessageBrokerType::RabbitMQ))
                .to_consumer_config()
                .is_ok()
        );
        assert!(sample_messaging_setting(None).to_consumer_config().is_err());
    }

    #[test]
    fn builds_producer_config_for_each_broker() {
        assert!(
            sample_messaging_setting(Some(MessageBrokerType::Kafka))
                .to_producer_config()
                .is_some()
        );
        assert!(
            sample_messaging_setting(Some(MessageBrokerType::Redis))
                .to_producer_config()
                .is_some()
        );
        assert!(
            sample_messaging_setting(Some(MessageBrokerType::RabbitMQ))
                .to_producer_config()
                .is_some()
        );
        assert!(
            sample_messaging_setting(None)
                .to_producer_config()
                .is_none()
        );
    }

    #[test]
    fn get_smtp_client_requires_credentials() {
        let mut setting = Setting::new();
        setting.smtp_user = None;
        setting.smtp_password = None;

        let error = setting.get_smtp_client().unwrap_err();

        assert!(
            error
                .to_string()
                .contains("SMTP_USER or SMTP_PASSWORD is not set in environment")
        );
    }

    #[test]
    fn get_smtp_client_builds_client_with_credentials() {
        let mut setting = Setting::new();
        setting.smtp_host = "localhost".to_string();
        setting.smtp_port = 1025;
        setting.smtp_tls = false;
        setting.smtp_user = Some("test@localhost".to_string());
        setting.smtp_password = Some("password".to_string());

        let client = setting.get_smtp_client();

        assert!(client.is_ok());
    }
}
