/// Tests for task helper functions
///
/// This module tests the helper functions in core::task module:
/// - publish_task: Publishing a task with default priority
/// - publish_task_with_priority: Publishing a task with custom priority
/// - publish_event: Internal helper for publishing TaskEvent
///
/// The tests use a MockProducer to verify task publishing without requiring
/// a real message broker.
use async_trait::async_trait;
use my_axum::{
    core::task::{TaskEvent, TaskPriority, TaskType, publish_task, publish_task_with_priority},
    pkg::messaging::MessageProducer,
};
use std::sync::{Arc, Mutex};

/// Mock producer for testing
#[derive(Clone)]
struct MockProducer {
    published_events: Arc<Mutex<Vec<String>>>,
    fail_on_publish: Arc<Mutex<bool>>,
}

impl MockProducer {
    fn new() -> Self {
        Self {
            published_events: Arc::new(Mutex::new(Vec::new())),
            fail_on_publish: Arc::new(Mutex::new(false)),
        }
    }

    fn get_published_events(&self) -> Vec<String> {
        self.published_events.lock().unwrap().clone()
    }

    fn set_fail_on_publish(&self, fail: bool) {
        *self.fail_on_publish.lock().unwrap() = fail;
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
        if *self.fail_on_publish.lock().unwrap() {
            return Err(anyhow::anyhow!("Mock publish failure"));
        }
        self.published_events
            .lock()
            .unwrap()
            .push(event_json.to_string());
        Ok(())
    }
}

mod publish_task_tests {
    use super::*;

    #[tokio::test]
    async fn test_publish_task_cleanup_expired_token() {
        let mock_producer = MockProducer::new();
        let task = TaskType::CleanupExpiredToken;

        let result = publish_task(&mock_producer, task.clone(), None).await;
        assert!(result.is_ok());

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 1);

        // Verify the event can be deserialized
        let event: TaskEvent = serde_json::from_str(&published[0]).unwrap();
        matches!(event.task, TaskType::CleanupExpiredToken);
        assert_eq!(event.priority, TaskPriority::Normal);
    }

    #[tokio::test]
    async fn test_publish_task_process_user_registration() {
        let mock_producer = MockProducer::new();
        let task = TaskType::ProcessUserRegistration { user_id: 123 };

        let result = publish_task(&mock_producer, task.clone(), None).await;
        assert!(result.is_ok());

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 1);

        let event: TaskEvent = serde_json::from_str(&published[0]).unwrap();
        match event.task {
            TaskType::ProcessUserRegistration { user_id } => {
                assert_eq!(user_id, 123);
            }
            _ => panic!("Expected ProcessUserRegistration task"),
        }
    }

    #[tokio::test]
    async fn test_publish_task_send_email() {
        let mock_producer = MockProducer::new();
        let task = TaskType::SendEmail {
            to: "test@example.com".to_string(),
            subject: "Test Subject".to_string(),
            text_body: Some("Text body".to_string()),
            html_body: Some("<p>HTML body</p>".to_string()),
        };

        let result = publish_task(&mock_producer, task.clone(), None).await;
        assert!(result.is_ok());

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 1);

        let event: TaskEvent = serde_json::from_str(&published[0]).unwrap();
        match event.task {
            TaskType::SendEmail {
                to,
                subject,
                text_body,
                html_body,
            } => {
                assert_eq!(to, "test@example.com");
                assert_eq!(subject, "Test Subject");
                assert_eq!(text_body, Some("Text body".to_string()));
                assert_eq!(html_body, Some("<p>HTML body</p>".to_string()));
            }
            _ => panic!("Expected SendEmail task"),
        }
    }

    #[tokio::test]
    async fn test_publish_task_with_destination() {
        let mock_producer = MockProducer::new();
        let task = TaskType::CleanupExpiredToken;

        let result = publish_task(&mock_producer, task.clone(), Some("custom-topic")).await;
        assert!(result.is_ok());

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 1);
    }

    #[tokio::test]
    async fn test_publish_task_producer_failure() {
        let mock_producer = MockProducer::new();
        mock_producer.set_fail_on_publish(true);

        let task = TaskType::CleanupExpiredToken;
        let result = publish_task(&mock_producer, task.clone(), None).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Mock publish failure")
        );
    }

    #[tokio::test]
    async fn test_publish_multiple_tasks_sequentially() {
        let mock_producer = MockProducer::new();

        // Publish first task
        let task1 = TaskType::CleanupExpiredToken;
        let result1 = publish_task(&mock_producer, task1, None).await;
        assert!(result1.is_ok());

        // Publish second task
        let task2 = TaskType::ProcessUserRegistration { user_id: 456 };
        let result2 = publish_task(&mock_producer, task2, None).await;
        assert!(result2.is_ok());

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 2);
    }
}

mod publish_task_with_priority_tests {
    use super::*;

    #[tokio::test]
    async fn test_publish_task_with_high_priority() {
        let mock_producer = MockProducer::new();
        let task = TaskType::SendEmail {
            to: "urgent@example.com".to_string(),
            subject: "Urgent".to_string(),
            text_body: None,
            html_body: None,
        };

        let result =
            publish_task_with_priority(&mock_producer, task.clone(), TaskPriority::High, None)
                .await;
        assert!(result.is_ok());

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 1);

        let event: TaskEvent = serde_json::from_str(&published[0]).unwrap();
        assert_eq!(event.priority, TaskPriority::High);
    }

    #[tokio::test]
    async fn test_publish_task_with_low_priority() {
        let mock_producer = MockProducer::new();
        let task = TaskType::CleanupExpiredToken;

        let result =
            publish_task_with_priority(&mock_producer, task.clone(), TaskPriority::Low, None).await;
        assert!(result.is_ok());

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 1);

        let event: TaskEvent = serde_json::from_str(&published[0]).unwrap();
        assert_eq!(event.priority, TaskPriority::Low);
    }

    #[tokio::test]
    async fn test_publish_task_with_normal_priority() {
        let mock_producer = MockProducer::new();
        let task = TaskType::ProcessUserRegistration { user_id: 789 };

        let result =
            publish_task_with_priority(&mock_producer, task.clone(), TaskPriority::Normal, None)
                .await;
        assert!(result.is_ok());

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 1);

        let event: TaskEvent = serde_json::from_str(&published[0]).unwrap();
        assert_eq!(event.priority, TaskPriority::Normal);
    }

    #[tokio::test]
    async fn test_publish_task_with_priority_and_destination() {
        let mock_producer = MockProducer::new();
        let task = TaskType::CleanupExpiredToken;

        let result = publish_task_with_priority(
            &mock_producer,
            task.clone(),
            TaskPriority::High,
            Some("priority-topic"),
        )
        .await;
        assert!(result.is_ok());

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 1);
    }

    #[tokio::test]
    async fn test_publish_task_with_priority_producer_failure() {
        let mock_producer = MockProducer::new();
        mock_producer.set_fail_on_publish(true);

        let task = TaskType::CleanupExpiredToken;
        let result =
            publish_task_with_priority(&mock_producer, task.clone(), TaskPriority::High, None)
                .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_publish_different_priorities() {
        let mock_producer = MockProducer::new();

        // Low priority
        let task1 = TaskType::CleanupExpiredToken;
        publish_task_with_priority(&mock_producer, task1, TaskPriority::Low, None)
            .await
            .unwrap();

        // Normal priority
        let task2 = TaskType::ProcessUserRegistration { user_id: 100 };
        publish_task_with_priority(&mock_producer, task2, TaskPriority::Normal, None)
            .await
            .unwrap();

        // High priority
        let task3 = TaskType::SendEmail {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            text_body: None,
            html_body: None,
        };
        publish_task_with_priority(&mock_producer, task3, TaskPriority::High, None)
            .await
            .unwrap();

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 3);

        // Verify priorities
        let event1: TaskEvent = serde_json::from_str(&published[0]).unwrap();
        assert_eq!(event1.priority, TaskPriority::Low);

        let event2: TaskEvent = serde_json::from_str(&published[1]).unwrap();
        assert_eq!(event2.priority, TaskPriority::Normal);

        let event3: TaskEvent = serde_json::from_str(&published[2]).unwrap();
        assert_eq!(event3.priority, TaskPriority::High);
    }
}

mod task_event_serialization_tests {
    use super::*;

    #[tokio::test]
    async fn test_task_event_roundtrip_serialization() {
        let mock_producer = MockProducer::new();
        let task = TaskType::SendEmail {
            to: "test@example.com".to_string(),
            subject: "Test".to_string(),
            text_body: Some("Body".to_string()),
            html_body: None,
        };

        publish_task(&mock_producer, task.clone(), None)
            .await
            .unwrap();

        let published = mock_producer.get_published_events();
        let event: TaskEvent = serde_json::from_str(&published[0]).unwrap();

        // Serialize again and compare
        let reserialized = serde_json::to_string(&event).unwrap();
        let event2: TaskEvent = serde_json::from_str(&reserialized).unwrap();

        // Verify all fields match
        match (&event.task, &event2.task) {
            (
                TaskType::SendEmail {
                    to: to1,
                    subject: s1,
                    text_body: tb1,
                    html_body: hb1,
                },
                TaskType::SendEmail {
                    to: to2,
                    subject: s2,
                    text_body: tb2,
                    html_body: hb2,
                },
            ) => {
                assert_eq!(to1, to2);
                assert_eq!(s1, s2);
                assert_eq!(tb1, tb2);
                assert_eq!(hb1, hb2);
            }
            _ => panic!("Task types don't match"),
        }

        assert_eq!(event.priority, event2.priority);
    }

    #[tokio::test]
    async fn test_task_event_with_all_task_types() {
        let mock_producer = MockProducer::new();

        // Test all task types can be serialized/deserialized
        let tasks = vec![
            TaskType::CleanupExpiredToken,
            TaskType::ProcessUserRegistration { user_id: 123 },
            TaskType::SendEmail {
                to: "test@example.com".to_string(),
                subject: "Test".to_string(),
                text_body: Some("Text".to_string()),
                html_body: Some("<p>HTML</p>".to_string()),
            },
        ];

        for task in tasks {
            publish_task(&mock_producer, task, None).await.unwrap();
        }

        let published = mock_producer.get_published_events();
        assert_eq!(published.len(), 3);

        // Verify all can be deserialized
        for event_json in published {
            let event: Result<TaskEvent, _> = serde_json::from_str(&event_json);
            assert!(event.is_ok());
        }
    }
}
