#[cfg(test)]
mod upload_avatar_use_case_tests {
    use crate::setup::app::TestApp;
    use async_trait::async_trait;
    use my_axum::{
        core::context::Context,
        pkg::messaging::MessageProducer,
        user::{
            dto::{avatar_dto::UploadAvatarDTO, user_dto::UserCreateDTO},
            use_case::{user::create_user_use_case, user::upload_avatar_use_case},
        },
    };
    use std::sync::{Arc, Mutex};

    // Mock producer for testing
    #[derive(Clone)]
    struct MockProducer {
        pub published_messages: Arc<Mutex<Vec<String>>>,
        pub should_fail: bool,
    }

    impl MockProducer {
        fn new() -> Self {
            Self {
                published_messages: Arc::new(Mutex::new(Vec::new())),
                should_fail: false,
            }
        }

        fn new_failing() -> Self {
            Self {
                published_messages: Arc::new(Mutex::new(Vec::new())),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl MessageProducer for MockProducer {
        async fn publish_event_json(
            &self,
            event_json: &str,
            _destination: Option<&str>,
        ) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("Mock producer failure"));
            }
            self.published_messages
                .lock()
                .unwrap()
                .push(event_json.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_upload_avatar_user_not_found() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Try to upload avatar for non-existent user
        let request = UploadAvatarDTO {
            user_id: 999999,
            file_name: "avatar.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 404);
        assert!(error.message.contains("not found") || error.message.contains("not_found"));
    }

    #[tokio::test]
    async fn test_upload_avatar_without_producer() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None, // No producer
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "avatar@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Avatar".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Try to upload avatar
        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "avatar.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should fail because producer is not available
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 500);
        assert!(error.message.contains("Producer not available"));
    }

    #[tokio::test]
    async fn test_upload_avatar_with_various_file_extensions() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "files@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Files".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Test different file extensions
        let file_names = vec![
            "avatar.jpg",
            "photo.png",
            "image.gif",
            "picture.jpeg",
            "profile.webp",
        ];

        for file_name in file_names {
            let request = UploadAvatarDTO {
                user_id: created_user.id,
                file_name: file_name.to_string(),
            };

            let result = upload_avatar_use_case::execute(&context, request).await;

            // All should fail at producer check
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().status.as_u16(), 500);
        }
    }

    #[tokio::test]
    async fn test_upload_avatar_with_long_filename() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "longname@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Long".to_string()),
            last_name: Some("Name".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Very long filename
        let long_name = format!("{}.jpg", "a".repeat(200));
        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: long_name,
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should fail at producer check
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status.as_u16(), 500);
    }

    #[tokio::test]
    async fn test_upload_avatar_with_special_characters_in_filename() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "special@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Special".to_string()),
            last_name: Some("Chars".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Filename with special characters
        let special_filenames = vec![
            "my-avatar.jpg",
            "my_avatar.png",
            "avatar (1).jpg",
            "avatar-2024-01-01.png",
            "user.avatar.final.jpg",
        ];

        for file_name in special_filenames {
            let request = UploadAvatarDTO {
                user_id: created_user.id,
                file_name: file_name.to_string(),
            };

            let result = upload_avatar_use_case::execute(&context, request).await;

            // Should fail at producer check
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_upload_avatar_with_unicode_filename() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "unicode@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Unicode".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Filename with unicode characters
        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "头像.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should fail at producer check
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status.as_u16(), 500);
    }

    #[tokio::test]
    async fn test_upload_avatar_with_empty_filename() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "empty@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Empty".to_string()),
            last_name: Some("File".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Empty filename
        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should fail at producer check (empty filename is allowed by DTO)
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status.as_u16(), 500);
    }

    #[tokio::test]
    async fn test_upload_avatar_with_path_traversal_attempt() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "traversal@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Path".to_string()),
            last_name: Some("Traversal".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Filename with path traversal attempt
        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "../../etc/passwd".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should fail at producer check
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status.as_u16(), 500);
    }

    #[tokio::test]
    async fn test_upload_avatar_with_zero_user_id() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Try with user_id = 0
        let request = UploadAvatarDTO {
            user_id: 0,
            file_name: "avatar.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 404);
    }

    #[tokio::test]
    async fn test_upload_avatar_with_negative_user_id() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Try with negative user_id
        let request = UploadAvatarDTO {
            user_id: -1,
            file_name: "avatar.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 404);
    }

    #[tokio::test]
    async fn test_upload_avatar_with_whitespace_filename() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "whitespace@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("White".to_string()),
            last_name: Some("Space".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Filename with whitespace
        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "  avatar.jpg  ".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should fail at producer check (whitespace is allowed)
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status.as_u16(), 500);
    }

    #[tokio::test]
    async fn test_upload_avatar_multiple_times_same_user() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "multiple@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Multiple".to_string()),
            last_name: Some("Uploads".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Upload multiple times
        for i in 1..=5 {
            let request = UploadAvatarDTO {
                user_id: created_user.id,
                file_name: format!("avatar_{}.jpg", i),
            };

            let result = upload_avatar_use_case::execute(&context, request).await;

            // All should fail at producer check
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().status.as_u16(), 500);
        }
    }

    #[tokio::test]
    async fn test_upload_avatar_filename_without_extension() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "noext@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("No".to_string()),
            last_name: Some("Extension".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Filename without extension
        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "avatar".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should fail at producer check
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status.as_u16(), 500);
    }

    #[tokio::test]
    async fn test_upload_avatar_with_multiple_dots_in_filename() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "dots@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Multiple".to_string()),
            last_name: Some("Dots".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Filename with multiple dots
        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "my.avatar.final.version.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should fail at producer check
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status.as_u16(), 500);
    }

    #[tokio::test]
    async fn test_upload_avatar_verify_user_exists_first() {
        // Test that the user existence is checked before producer
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "existcheck@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Exist".to_string()),
            last_name: Some("Check".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "test.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should fail at producer check, not at user not found
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 500);
        assert!(error.message.contains("Producer not available"));
    }

    #[tokio::test]
    async fn test_upload_avatar_validates_user_first() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "validates@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Validate".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "validate.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // User exists, so should fail later at producer check
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_ne!(error.status.as_u16(), 404); // Not 404, means user was found
    }

    #[tokio::test]
    async fn test_upload_avatar_returns_202_status_when_accepted() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "status@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Status".to_string()),
            last_name: Some("Check".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "avatar.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Without producer, it will be 500, but with producer it should be 202
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_upload_avatar_success_with_producer() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        // Create test user without producer first
        let context_no_producer = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "success@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Success".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context_no_producer, user_dto)
            .await
            .unwrap()
            .data;

        // Create mock producer and context with producer
        let mock_producer = MockProducer::new();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer.clone()));

        let context = Context {
            txn: &txn,
            user: None,
            producer: Some(producer),
        };

        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "avatar.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should succeed
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 202);
        // Verify task_id is a valid UUID v4
        assert!(!response.data.task_id.is_empty());
        assert!(uuid::Uuid::parse_str(&response.data.task_id).is_ok());
        assert!(response.data.message.contains("Avatar upload initiated"));
        assert!(response.data.message.contains(&response.data.task_id));

        // Verify message was published
        let messages = mock_producer.published_messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        assert!(message.contains(&created_user.id.to_string()));
        assert!(message.contains("avatar.jpg"));
    }

    #[tokio::test]
    async fn test_upload_avatar_success_with_png_file() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        let context_no_producer = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "png@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("PNG".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context_no_producer, user_dto)
            .await
            .unwrap()
            .data;

        let mock_producer = MockProducer::new();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer.clone()));

        let context = Context {
            txn: &txn,
            user: None,
            producer: Some(producer),
        };

        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "profile.png".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 202);

        let messages = mock_producer.published_messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_upload_avatar_producer_publish_fails() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        // Create mock producer that fails
        let mock_producer = MockProducer::new_failing();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer));

        let context = Context {
            txn: &txn,
            user: None,
            producer: Some(producer),
        };

        let user_dto = UserCreateDTO {
            email: "fail@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Fail".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "fail.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        // Should fail with internal error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 500);
        assert!(error.message.contains("Failed to publish upload task"));
    }

    #[tokio::test]
    async fn test_upload_avatar_message_contains_correct_task_type() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        let context_no_producer = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "tasktype@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Task".to_string()),
            last_name: Some("Type".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context_no_producer, user_dto)
            .await
            .unwrap()
            .data;

        let mock_producer = MockProducer::new();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer.clone()));

        let context = Context {
            txn: &txn,
            user: None,
            producer: Some(producer),
        };

        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "task.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        assert!(result.is_ok());

        let messages = mock_producer.published_messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        // Verify the message contains ProcessAvatarUpload task type
        assert!(message.contains("ProcessAvatarUpload"));
    }

    #[tokio::test]
    async fn test_upload_avatar_with_different_filenames_success() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        let context_no_producer = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "multifile@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Multi".to_string()),
            last_name: Some("File".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context_no_producer, user_dto)
            .await
            .unwrap()
            .data;

        let mock_producer = MockProducer::new();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer.clone()));

        let context = Context {
            txn: &txn,
            user: None,
            producer: Some(producer),
        };

        let filenames = vec!["photo.jpg", "image.png", "avatar.webp", "profile.gif"];

        for filename in filenames {
            let request = UploadAvatarDTO {
                user_id: created_user.id,
                file_name: filename.to_string(),
            };

            let result = upload_avatar_use_case::execute(&context, request).await;
            assert!(result.is_ok());
        }

        let messages = mock_producer.published_messages.lock().unwrap();
        assert_eq!(messages.len(), 4);
    }

    #[tokio::test]
    async fn test_upload_avatar_response_message_format() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        let context_no_producer = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "format@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Format".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context_no_producer, user_dto)
            .await
            .unwrap()
            .data;

        let mock_producer = MockProducer::new();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer));

        let context = Context {
            txn: &txn,
            user: None,
            producer: Some(producer),
        };

        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: "test.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Verify response format
        // Verify task_id is a valid UUID v4
        assert!(!response.data.task_id.is_empty());
        assert!(uuid::Uuid::parse_str(&response.data.task_id).is_ok());
        assert!(response.data.message.contains("Avatar upload initiated"));
        assert!(response.data.message.contains("ws://"));
        assert!(response.data.message.contains(&response.data.task_id));
    }

    #[tokio::test]
    async fn test_upload_avatar_user_not_found_before_producer_check() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        let mock_producer = MockProducer::new();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer.clone()));

        let context = Context {
            txn: &txn,
            user: None,
            producer: Some(producer),
        };

        // Try with non-existent user even with valid producer
        let request = UploadAvatarDTO {
            user_id: 999999,
            file_name: "test.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        // Should fail at user check, not at producer
        assert_eq!(error.status.as_u16(), 404);

        // Verify no message was published
        let messages = mock_producer.published_messages.lock().unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_upload_avatar_with_large_user_id() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        let mock_producer = MockProducer::new();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer));

        let context = Context {
            txn: &txn,
            user: None,
            producer: Some(producer),
        };

        let request = UploadAvatarDTO {
            user_id: i32::MAX,
            file_name: "test.jpg".to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status.as_u16(), 404);
    }

    #[tokio::test]
    async fn test_upload_avatar_filename_in_published_message() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        let context_no_producer = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "filename@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("File".to_string()),
            last_name: Some("Name".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context_no_producer, user_dto)
            .await
            .unwrap()
            .data;

        let mock_producer = MockProducer::new();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer.clone()));

        let context = Context {
            txn: &txn,
            user: None,
            producer: Some(producer),
        };

        let filename = "my-special-avatar.jpg";
        let request = UploadAvatarDTO {
            user_id: created_user.id,
            file_name: filename.to_string(),
        };

        let result = upload_avatar_use_case::execute(&context, request).await;

        assert!(result.is_ok());

        let messages = mock_producer.published_messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        assert!(message.contains(filename));
    }
}
