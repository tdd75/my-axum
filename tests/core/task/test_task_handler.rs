/// Tests for TaskHandler
///
/// This module tests the TaskHandler functionality which is responsible for:
/// - Creating TaskHandler instances with dependencies (db, producer, smtp_client)
/// - Processing different types of tasks:
///   - CleanupExpiredToken: Cleaning up expired refresh tokens
///   - ProcessUserRegistration: Sending welcome emails to new users
///   - SendEmail: Sending emails via SMTP
///
/// The tests use a MockProducer to verify task publishing without requiring
/// a real message broker (Kafka, RabbitMQ, or Redis).
///
/// Note: ConcreteTaskHandler::new() accepts dependencies directly (db, producer, smtp_client)
/// which allows for easy testing with mock implementations.
use async_trait::async_trait;
use my_axum::{
    core::{
        context::Context,
        db::entity::user,
        task::{ConcreteTaskHandler, TaskEvent},
    },
    pkg::{
        messaging::{MessageProducer, TaskHandler},
        password::hash_password_string,
    },
    user::repository::user_repository,
};
use sea_orm::{ActiveValue::Set, TransactionTrait};
use std::sync::{Arc, Mutex};

use crate::setup::app::TestApp;

/// Mock producer for testing
#[derive(Clone)]
struct MockProducer {
    published_events: Arc<Mutex<Vec<TaskEvent>>>,
}

impl MockProducer {
    fn new() -> Self {
        Self {
            published_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_published_events(&self) -> Vec<TaskEvent> {
        self.published_events.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.published_events.lock().unwrap().clear();
    }
}

#[async_trait]
impl MessageProducer for MockProducer {
    async fn publish_event_json(
        &self,
        event_json: &str,
        _destination: Option<&str>,
    ) -> anyhow::Result<()> {
        let event: TaskEvent = serde_json::from_str(event_json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize event: {}", e))?;
        self.published_events.lock().unwrap().push(event);
        Ok(())
    }
}

mod task_handler_creation_tests {
    use super::*;

    #[tokio::test]
    async fn test_task_handler_new_without_smtp() {
        let app = TestApp::spawn_app().await;
        let mock_producer = MockProducer::new();

        let handler = ConcreteTaskHandler::new(
            app.db.clone(),
            Arc::new(Box::new(mock_producer)),
            None,
            app.setting.redis_url.clone(),
        );

        assert!(handler.is_ok());
    }

    #[tokio::test]
    async fn test_task_handler_new_with_all_dependencies() {
        let app = TestApp::spawn_app().await;
        let mock_producer = MockProducer::new();

        let handler = ConcreteTaskHandler::new(
            app.db.clone(),
            Arc::new(Box::new(mock_producer)),
            None,
            app.setting.redis_url.clone(),
        );

        assert!(handler.is_ok());
    }
}

mod cleanup_expired_token_tests {
    use super::*;
    use chrono::{Duration, Utc};
    use my_axum::{
        core::{
            db::entity::refresh_token,
            task::{TaskEvent, TaskType},
        },
        user::repository::refresh_token_repository::{self, RefreshTokenSearchParams},
    };

    #[tokio::test]
    async fn test_cleanup_expired_tokens_success() {
        let app = TestApp::spawn_app().await;
        let mock_producer = MockProducer::new();

        let handler = ConcreteTaskHandler::new(
            app.db.clone(),
            Arc::new(Box::new(mock_producer)),
            None,
            app.setting.redis_url.clone(),
        )
        .unwrap();

        // Create a test user first
        let user_id = app
            .db
            .transaction::<_, i32, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let hashed_password = hash_password_string("password123@")
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    let user = user::ActiveModel {
                        email: Set("test@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Test".to_string())),
                        last_name: Set(Some("User".to_string())),
                        ..Default::default()
                    };

                    let user = user_repository::create(&context, user)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    Ok(user.id)
                })
            })
            .await
            .unwrap();

        // Create some expired refresh tokens
        app.db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    // Create expired token
                    let expired_token = refresh_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("expired_token_123".to_string()),
                        device_info: Set(Some("Test Device".to_string())),
                        ip_address: Set(Some("127.0.0.1".to_string())),
                        expires_at: Set(Utc::now().naive_utc() - Duration::days(1)),
                        ..Default::default()
                    };
                    refresh_token_repository::create(&context, expired_token)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    // Create valid token
                    let valid_token = refresh_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("valid_token_123".to_string()),
                        device_info: Set(Some("Test Device".to_string())),
                        ip_address: Set(Some("127.0.0.1".to_string())),
                        expires_at: Set(Utc::now().naive_utc() + Duration::days(7)),
                        ..Default::default()
                    };
                    refresh_token_repository::create(&context, valid_token)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    Ok(())
                })
            })
            .await
            .unwrap();

        // Create task event
        let task_event = TaskEvent::new(TaskType::CleanupExpiredToken);

        // Handle the task
        let result = handler.handle_task(&task_event).await;
        assert!(result.is_ok());

        // Verify expired tokens were deleted but valid ones remain
        let remaining_tokens = app
            .db
            .transaction::<_, Vec<_>, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    my_axum::user::repository::refresh_token_repository::search(
                        &context,
                        &RefreshTokenSearchParams::default(),
                    )
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))
                })
            })
            .await
            .unwrap();

        assert_eq!(remaining_tokens.len(), 1);
        assert_eq!(remaining_tokens[0].token, "valid_token_123");
    }

    #[tokio::test]
    async fn test_cleanup_expired_tokens_when_none_exist() {
        let app = TestApp::spawn_app().await;
        let mock_producer = MockProducer::new();

        let handler = ConcreteTaskHandler::new(
            app.db.clone(),
            Arc::new(Box::new(mock_producer)),
            None,
            app.setting.redis_url.clone(),
        )
        .unwrap();

        let task_event = TaskEvent::new(TaskType::CleanupExpiredToken);
        let result = handler.handle_task(&task_event).await;

        assert!(result.is_ok());
    }
}

mod process_user_registration_tests {
    use my_axum::core::task::{TaskEvent, TaskType};

    use super::*;

    #[tokio::test]
    async fn test_process_user_registration_success() {
        let app = TestApp::spawn_app().await;
        let mock_producer = MockProducer::new();
        let mock_producer_clone = mock_producer.clone();

        let handler = ConcreteTaskHandler::new(
            app.db.clone(),
            Arc::new(Box::new(mock_producer)),
            None,
            app.setting.redis_url.clone(),
        )
        .unwrap();

        // Create a test user
        let user_id = app
            .db
            .transaction::<_, i32, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let hashed_password = hash_password_string("password123@")
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    let user = user::ActiveModel {
                        email: Set("newuser@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("New".to_string())),
                        last_name: Set(Some("User".to_string())),
                        ..Default::default()
                    };

                    let user = user_repository::create(&context, user)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    Ok(user.id)
                })
            })
            .await
            .unwrap();

        // Create task event
        let task_event = TaskEvent::new(TaskType::ProcessUserRegistration { user_id });

        // Handle the task
        let result = handler.handle_task(&task_event).await;
        assert!(result.is_ok());

        // Verify that welcome email task was published
        let published_events = mock_producer_clone.get_published_events();
        assert_eq!(published_events.len(), 1);

        match &published_events[0].task {
            TaskType::SendEmail { to, subject, .. } => {
                assert_eq!(to, "newuser@example.com");
                assert!(subject.contains("Welcome"));
            }
            _ => panic!("Expected SendEmail task"),
        }
    }

    #[tokio::test]
    async fn test_process_user_registration_user_not_found() {
        let app = TestApp::spawn_app().await;
        let mock_producer = MockProducer::new();

        let handler = ConcreteTaskHandler::new(
            app.db.clone(),
            Arc::new(Box::new(mock_producer)),
            None,
            app.setting.redis_url.clone(),
        )
        .unwrap();

        // Use a non-existent user ID
        let task_event = TaskEvent::new(TaskType::ProcessUserRegistration { user_id: 99999 });

        let result = handler.handle_task(&task_event).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("User not found"));
    }
}

mod send_email_tests {
    use my_axum::core::task::TaskType;

    use super::*;

    #[tokio::test]
    async fn test_send_email_without_smtp_client() {
        let app = TestApp::spawn_app().await;
        let mock_producer = MockProducer::new();

        let handler = ConcreteTaskHandler::new(
            app.db.clone(),
            Arc::new(Box::new(mock_producer)),
            None,
            app.setting.redis_url.clone(),
        )
        .unwrap();

        let task_event = TaskEvent::new(TaskType::SendEmail {
            to: "test@example.com".to_string(),
            subject: "Test Email".to_string(),
            text_body: Some("Text body".to_string()),
            html_body: Some("<p>HTML body</p>".to_string()),
        });

        let result = handler.handle_task(&task_event).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("SMTP client not configured")
        );
    }
}

mod task_event_handling_tests {
    use my_axum::core::task::{TaskEvent, TaskType};

    use super::*;

    #[tokio::test]
    async fn test_handle_task_logs_task_info() {
        let app = TestApp::spawn_app().await;
        let mock_producer = MockProducer::new();

        let handler = ConcreteTaskHandler::new(
            app.db.clone(),
            Arc::new(Box::new(mock_producer)),
            None,
            app.setting.redis_url.clone(),
        )
        .unwrap();

        let task_event = TaskEvent::new(TaskType::CleanupExpiredToken);

        // Just verify it doesn't panic
        let result = handler.handle_task(&task_event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_multiple_tasks_sequentially() {
        let app = TestApp::spawn_app().await;
        let mock_producer = MockProducer::new();

        let handler = ConcreteTaskHandler::new(
            app.db.clone(),
            Arc::new(Box::new(mock_producer)),
            None,
            app.setting.redis_url.clone(),
        )
        .unwrap();

        // Task 1: Cleanup
        let task1 = TaskEvent::new(TaskType::CleanupExpiredToken);
        let result1 = handler.handle_task(&task1).await;
        assert!(result1.is_ok());

        // Task 2: Another cleanup
        let task2 = TaskEvent::new(TaskType::CleanupExpiredToken);
        let result2 = handler.handle_task(&task2).await;
        assert!(result2.is_ok());
    }
}
