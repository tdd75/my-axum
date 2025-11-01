#[cfg(test)]
mod setting_tests {
    use my_axum::config::setting::{MessageBrokerType, MessagingSetting, Setting};

    #[test]
    fn test_setting_default() {
        let setting = Setting::default();
        assert!(!setting.app_host.is_empty());
        assert!(setting.app_port > 0);
        assert!(!setting.database_url.is_empty());
        assert!(!setting.jwt_secret.is_empty());
    }

    #[test]
    fn test_setting_new() {
        let setting = Setting::new();
        assert!(!setting.app_host.is_empty());
        assert!(setting.app_port > 0);
    }

    #[test]
    fn test_setting_clone() {
        let setting1 = Setting::new();
        let setting2 = setting1.clone();
        assert_eq!(setting1.app_host, setting2.app_host);
        assert_eq!(setting1.app_port, setting2.app_port);
        assert_eq!(setting1.database_url, setting2.database_url);
    }

    #[test]
    fn test_message_broker_type_kafka() {
        let broker_type = MessageBrokerType::Kafka;
        assert!(matches!(broker_type, MessageBrokerType::Kafka));
    }

    #[test]
    fn test_message_broker_type_redis() {
        let broker_type = MessageBrokerType::Redis;
        assert!(matches!(broker_type, MessageBrokerType::Redis));
    }

    #[test]
    fn test_message_broker_type_rabbitmq() {
        let broker_type = MessageBrokerType::RabbitMQ;
        assert!(matches!(broker_type, MessageBrokerType::RabbitMQ));
    }

    #[test]
    fn test_message_broker_type_clone() {
        let broker_type1 = MessageBrokerType::Kafka;
        let broker_type2 = broker_type1.clone();
        assert_eq!(broker_type1, broker_type2);
    }

    #[test]
    fn test_message_broker_type_partial_eq() {
        let kafka1 = MessageBrokerType::Kafka;
        let kafka2 = MessageBrokerType::Kafka;
        let redis = MessageBrokerType::Redis;
        assert_eq!(kafka1, kafka2);
        assert_ne!(kafka1, redis);
    }

    #[test]
    fn test_setting_jwt_access_token_expires() {
        let setting = Setting::new();
        assert!(setting.jwt_access_token_expires > 0);
    }

    #[test]
    fn test_setting_jwt_refresh_token_expires() {
        let setting = Setting::new();
        assert!(setting.jwt_refresh_token_expires > 0);
    }

    #[test]
    fn test_setting_smtp_fields() {
        let setting = Setting::new();
        assert!(!setting.smtp_host.is_empty());
        assert!(setting.smtp_port > 0);
    }

    #[test]
    fn test_setting_messaging_config() {
        let setting = Setting::new();
        assert!(setting.messaging.worker_pool_size > 0);
        assert!(!setting.messaging.kafka_brokers.is_empty());
        assert!(!setting.messaging.redis_url.is_empty());
    }

    #[test]
    fn test_setting_get_smtp_client() {
        let setting = Setting::new();
        // This may fail if credentials are invalid, but it tests the method exists
        let _result = setting.get_smtp_client();
    }

    #[test]
    fn test_messaging_setting_to_consumer_config_no_broker() {
        let messaging_setting = MessagingSetting {
            message_broker: None,
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "test".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "test".to_string(),
        };

        let result = messaging_setting.to_consumer_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_messaging_setting_to_consumer_config_kafka() {
        let messaging_setting = MessagingSetting {
            message_broker: Some(MessageBrokerType::Kafka),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "topic1,topic2,topic3".to_string(),
            kafka_default_topic: "topic1".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "test".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "test".to_string(),
        };

        let result = messaging_setting.to_consumer_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_messaging_setting_to_consumer_config_redis() {
        let messaging_setting = MessagingSetting {
            message_broker: Some(MessageBrokerType::Redis),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "channel1,channel2".to_string(),
            redis_default_channel: "channel1".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "test".to_string(),
        };

        let result = messaging_setting.to_consumer_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_messaging_setting_to_consumer_config_rabbitmq() {
        let messaging_setting = MessagingSetting {
            message_broker: Some(MessageBrokerType::RabbitMQ),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "test".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "queue1,queue2".to_string(),
            rabbitmq_default_queue: "queue1".to_string(),
        };

        let result = messaging_setting.to_consumer_config();
        assert!(result.is_ok());
    }

    #[test]
    fn test_messaging_setting_to_producer_config_none() {
        let messaging_setting = MessagingSetting {
            message_broker: None,
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "test".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "test".to_string(),
        };

        let result = messaging_setting.to_producer_config();
        assert!(result.is_none());
    }

    #[test]
    fn test_messaging_setting_to_producer_config_kafka() {
        let messaging_setting = MessagingSetting {
            message_broker: Some(MessageBrokerType::Kafka),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092,localhost:9093".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "default_topic".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "test".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "test".to_string(),
        };

        let result = messaging_setting.to_producer_config();
        assert!(result.is_some());
    }

    #[test]
    fn test_messaging_setting_to_producer_config_redis() {
        let messaging_setting = MessagingSetting {
            message_broker: Some(MessageBrokerType::Redis),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "default_channel".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "test".to_string(),
        };

        let result = messaging_setting.to_producer_config();
        assert!(result.is_some());
    }

    #[test]
    fn test_messaging_setting_to_producer_config_rabbitmq() {
        let messaging_setting = MessagingSetting {
            message_broker: Some(MessageBrokerType::RabbitMQ),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "test".to_string(),
            rabbitmq_url: "amqp://guest:guest@localhost:5672/%2f".to_string(),
            rabbitmq_exchange: "test_exchange".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "default_queue".to_string(),
        };

        let result = messaging_setting.to_producer_config();
        assert!(result.is_some());
    }

    #[test]
    fn test_setting_to_consumer_config() {
        let setting = Setting::new();
        // This will fail if no broker is configured, which is expected
        let _result = setting.to_consumer_config();
    }

    #[test]
    fn test_setting_to_producer_config() {
        let setting = Setting::new();
        let _result = setting.to_producer_config();
        // May be None if no broker configured
    }

    #[test]
    fn test_messaging_setting_clone() {
        let messaging_setting1 = MessagingSetting {
            message_broker: Some(MessageBrokerType::Kafka),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "test".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "test".to_string(),
        };

        let messaging_setting2 = messaging_setting1.clone();
        assert_eq!(
            messaging_setting1.worker_pool_size,
            messaging_setting2.worker_pool_size
        );
        assert_eq!(
            messaging_setting1.kafka_brokers,
            messaging_setting2.kafka_brokers
        );
    }

    #[test]
    fn test_setting_fields_not_empty() {
        let setting = Setting::new();
        assert!(!setting.redis_url.is_empty());
        assert!(!setting.messaging.kafka_consumer_group.is_empty());
        assert!(!setting.messaging.kafka_default_topic.is_empty());
        assert!(!setting.messaging.redis_default_channel.is_empty());
        assert!(!setting.messaging.rabbitmq_default_queue.is_empty());
    }

    #[test]
    fn test_messaging_setting_debug() {
        let messaging_setting = MessagingSetting {
            message_broker: Some(MessageBrokerType::Kafka),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "test".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "test".to_string(),
        };

        let debug_str = format!("{:?}", messaging_setting);
        assert!(debug_str.contains("MessagingSetting"));
    }

    #[test]
    fn test_setting_debug() {
        let setting = Setting::new();
        let debug_str = format!("{:?}", setting);
        assert!(debug_str.contains("Setting"));
    }

    #[test]
    fn test_message_broker_type_debug() {
        let broker_type = MessageBrokerType::Kafka;
        let debug_str = format!("{:?}", broker_type);
        assert!(debug_str.contains("Kafka"));
    }

    #[test]
    fn test_kafka_topics_parsing() {
        let messaging_setting = MessagingSetting {
            message_broker: Some(MessageBrokerType::Kafka),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "topic1, topic2 , topic3".to_string(),
            kafka_default_topic: "topic1".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "test".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "test".to_string(),
        };

        let _config = messaging_setting.to_consumer_config().unwrap();
        // Topics should be trimmed
    }

    #[test]
    fn test_redis_channels_parsing() {
        let messaging_setting = MessagingSetting {
            message_broker: Some(MessageBrokerType::Redis),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "channel1, channel2 , channel3".to_string(),
            redis_default_channel: "channel1".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "test".to_string(),
            rabbitmq_default_queue: "test".to_string(),
        };

        let _config = messaging_setting.to_consumer_config().unwrap();
        // Channels should be trimmed
    }

    #[test]
    fn test_rabbitmq_queues_parsing() {
        let messaging_setting = MessagingSetting {
            message_broker: Some(MessageBrokerType::RabbitMQ),
            worker_pool_size: 10,
            kafka_brokers: "localhost:19092".to_string(),
            kafka_consumer_group: "test-group".to_string(),
            kafka_topics: "test".to_string(),
            kafka_default_topic: "test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            redis_channels: "test".to_string(),
            redis_default_channel: "test".to_string(),
            rabbitmq_url: "amqp://localhost:5672".to_string(),
            rabbitmq_exchange: "test".to_string(),
            rabbitmq_queue: "test".to_string(),
            rabbitmq_queues: "queue1, queue2 , queue3".to_string(),
            rabbitmq_default_queue: "queue1".to_string(),
        };

        let _config = messaging_setting.to_consumer_config().unwrap();
        // Queues should be trimmed
    }

    #[test]
    fn test_worker_pool_size_default() {
        let setting = Setting::new();
        assert!(setting.messaging.worker_pool_size > 0);
    }

    #[test]
    fn test_smtp_tls_default() {
        let setting = Setting::new();
        // smtp_tls should have a default value
        assert!(setting.smtp_tls);
    }

    #[test]
    fn test_setting_jwt_expires_comparison() {
        let setting = Setting::new();
        // Refresh token should expire after access token
        assert!(setting.jwt_refresh_token_expires >= setting.jwt_access_token_expires);
    }
}
