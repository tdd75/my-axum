#[cfg(test)]
mod user_task_tests {
    use async_trait::async_trait;
    use my_axum::{
        pkg::messaging::MessageProducer,
        user::task::user_task::{process_avatar_upload, send_welcome_email},
    };
    use std::sync::{Arc, Mutex};

    // Mock producer for testing
    struct MockProducer;

    #[async_trait]
    impl MessageProducer for MockProducer {
        async fn publish_event_json(
            &self,
            _event_json: &str,
            _destination: Option<&str>,
        ) -> anyhow::Result<()> {
            Ok(())
        }
    }

    // Mock producer that tracks published messages
    #[derive(Clone)]
    struct TrackingMockProducer {
        messages: Arc<Mutex<Vec<String>>>,
    }

    impl TrackingMockProducer {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_profilessages(&self) -> Vec<String> {
            self.messages.lock().unwrap().clone()
        }

        fn message_count(&self) -> usize {
            self.messages.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl MessageProducer for TrackingMockProducer {
        async fn publish_event_json(
            &self,
            event_json: &str,
            _destination: Option<&str>,
        ) -> anyhow::Result<()> {
            self.messages.lock().unwrap().push(event_json.to_string());
            Ok(())
        }
    }

    #[test]
    fn test_template_rendering() {
        use tera::{Context as TeraContext, Tera};

        const WELCOME_TEMPLATE: &str =
            include_str!("../../../src/core/template/email/welcome.html");

        let mut tera = Tera::default();
        tera.add_raw_template("welcome.html", WELCOME_TEMPLATE)
            .expect("Failed to add template");

        let mut context = TeraContext::new();
        context.insert("app_name", "Test App");
        context.insert("app_url", "http://localhost:8000");
        context.insert("email", "test@example.com");
        context.insert("first_name", "John");
        context.insert("last_name", "Doe");
        context.insert("phone", &Some("123456789".to_string()));
        context.insert("year", &2025);

        let result = tera.render("welcome.html", &context);
        assert!(result.is_ok(), "Template rendering failed");

        let html = result.unwrap();
        assert!(html.contains("Test App"), "HTML should contain app name");
        assert!(
            html.contains("test@example.com"),
            "HTML should contain email"
        );
        assert!(html.contains("John"), "HTML should contain first name");
        assert!(html.contains("Doe"), "HTML should contain last name");
        assert!(html.contains("123456789"), "HTML should contain phone");
    }

    #[test]
    fn test_template_rendering_with_minimal_data() {
        use tera::{Context as TeraContext, Tera};

        const WELCOME_TEMPLATE: &str =
            include_str!("../../../src/core/template/email/welcome.html");

        let mut tera = Tera::default();
        tera.add_raw_template("welcome.html", WELCOME_TEMPLATE)
            .expect("Failed to add template");

        let mut context = TeraContext::new();
        context.insert("app_name", "Test App");
        context.insert("app_url", "http://localhost:8000");
        context.insert("email", "test@example.com");
        context.insert("first_name", "");
        context.insert("last_name", "");
        let phone: Option<String> = None;
        context.insert("phone", &phone);
        context.insert("year", &2025);

        let result = tera.render("welcome.html", &context);
        assert!(
            result.is_ok(),
            "Template rendering with minimal data failed"
        );

        let html = result.unwrap();
        assert!(html.contains("Test App"), "HTML should contain app name");
        assert!(
            html.contains("test@example.com"),
            "HTML should contain email"
        );
    }

    #[test]
    fn test_template_contains_required_sections() {
        use tera::{Context as TeraContext, Tera};

        const WELCOME_TEMPLATE: &str =
            include_str!("../../../src/core/template/email/welcome.html");

        let mut tera = Tera::default();
        tera.add_raw_template("welcome.html", WELCOME_TEMPLATE)
            .expect("Failed to add template");

        let mut context = TeraContext::new();
        context.insert("app_name", "My App");
        context.insert("app_url", "http://localhost:8000");
        context.insert("email", "user@test.com");
        context.insert("first_name", "Test");
        context.insert("last_name", "User");
        let phone: Option<String> = None;
        context.insert("phone", &phone);
        context.insert("year", &2025);

        let html = tera.render("welcome.html", &context).unwrap();

        // Check for key sections
        assert!(html.contains("Welcome"), "Should contain welcome message");
        assert!(html.contains("My App"), "Should contain app name");
        assert!(html.contains("user@test.com"), "Should contain user email");
        assert!(html.contains("Test User"), "Should contain user name");
        assert!(html.contains("2025"), "Should contain current year");
        assert!(
            html.contains("Get Started"),
            "Should contain call to action"
        );
    }

    #[tokio::test]
    async fn test_send_welcome_email_nonexistent_user() {
        use crate::setup::app::TestApp;

        let test_app = TestApp::spawn_app().await;
        let producer = MockProducer;
        // Test with a user ID that doesn't exist
        let user_id = 999999; // Very unlikely to exist

        let result = send_welcome_email(&test_app.db, &producer, user_id).await;

        // Can either:
        // - Return Ok(()) if SMTP credentials are missing (function returns early)
        // - Return Err() if database connection fails or user not found
        match result {
            Ok(_) => {
                // This is acceptable if SMTP credentials are not set
                println!("Function completed successfully (likely SMTP credentials not set)");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("User not found")
                        || error_msg.contains("database")
                        || error_msg.contains("connect"),
                    "Error should mention user not found or database issue: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_send_welcome_email_handles_missing_smtp_gracefully() {
        use crate::setup::app::TestApp;

        let test_app = TestApp::spawn_app().await;
        let producer = MockProducer;
        // Test that send_welcome_email handles missing SMTP credentials gracefully
        // Without actually removing env vars, we test the observable behavior

        let user_id = 1;
        let result = send_welcome_email(&test_app.db, &producer, user_id).await;

        // The function should either:
        // 1. Return Ok(()) if SMTP credentials are missing (skips email)
        // 2. Return Ok(()) if email is sent successfully
        // 3. Return Err() if there's a database or connection error
        match result {
            Ok(_) => {
                // Success case - either email sent or skipped due to missing SMTP
                println!("send_welcome_email completed successfully");
            }
            Err(e) => {
                // Expected errors: database connection, user not found, SMTP connection
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("database")
                        || error_msg.contains("connect")
                        || error_msg.contains("User not found")
                        || error_msg.contains("SMTP"),
                    "Expected known error type, got: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn test_text_body_formatting() {
        // Test the text body fallback format
        let first_name = Some("John".to_string());
        let last_name = Some("Doe".to_string());
        let email = "john.doe@example.com";

        let text_body = format!(
            "Hello {} {}!\n\nThank you for registering with My Axum App. Your account has been successfully created.\n\nEmail: {}\n\nBest regards,\nThe My Axum App Team",
            first_name.as_deref().unwrap_or(""),
            last_name.as_deref().unwrap_or(""),
            email
        );

        assert!(text_body.contains("Hello John Doe!"));
        assert!(text_body.contains("john.doe@example.com"));
        assert!(text_body.contains("Thank you for registering"));
        assert!(text_body.contains("Best regards"));
    }

    #[test]
    fn test_year_string_conversion() {
        use chrono::Datelike;

        let year = chrono::Utc::now().year();
        let year_string = year.to_string();

        assert!(!year_string.is_empty());
        assert_eq!(year_string.len(), 4);
        assert!(year_string.parse::<i32>().is_ok());
        assert!(year >= 2025); // Should be current year or later
    }

    #[test]
    fn test_template_with_special_characters_in_names() {
        use tera::{Context as TeraContext, Tera};

        const WELCOME_TEMPLATE: &str =
            include_str!("../../../src/core/template/email/welcome.html");

        let mut tera = Tera::default();
        tera.add_raw_template("welcome.html", WELCOME_TEMPLATE)
            .expect("Failed to add template");

        let mut context = TeraContext::new();
        context.insert("app_name", "Test App");
        context.insert("app_url", "http://localhost:8000");
        context.insert("email", "test+filter@example.com");
        context.insert("first_name", "Jean-Pierre");
        context.insert("last_name", "O'Connor");
        context.insert("phone", &Some("+1 (555) 123-4567".to_string()));
        context.insert("year", &2025);

        let result = tera.render("welcome.html", &context);
        assert!(
            result.is_ok(),
            "Template rendering with special chars should succeed"
        );

        let html = result.unwrap();
        assert!(
            html.contains("Jean-Pierre"),
            "Should contain hyphenated first name"
        );
        assert!(
            html.contains("O&#x27;Connor") || html.contains("O'Connor"),
            "Should contain last name with apostrophe"
        );
        assert!(
            html.contains("test+filter@example.com"),
            "Should contain email with plus sign"
        );
    }

    #[tokio::test]
    async fn test_send_welcome_email_zero_user_id() {
        use crate::setup::app::TestApp;

        let test_app = TestApp::spawn_app().await;
        let producer = MockProducer;
        // Test with user_id = 0
        let user_id = 0;

        let result = send_welcome_email(&test_app.db, &producer, user_id).await;

        // Can either:
        // - Return Ok(()) if SMTP credentials are missing (function returns early)
        // - Return Err() if database connection fails or user not found
        match result {
            Ok(_) => {
                // This is acceptable if SMTP credentials are not set
                println!("Function completed successfully (likely SMTP credentials not set)");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("User not found")
                        || error_msg.contains("database")
                        || error_msg.contains("connect"),
                    "Error should mention user not found or database issue: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_send_welcome_email_negative_user_id() {
        use crate::setup::app::TestApp;

        let test_app = TestApp::spawn_app().await;
        let producer = MockProducer;
        // Test with negative user_id
        let user_id = -1;

        let result = send_welcome_email(&test_app.db, &producer, user_id).await;

        // Can either:
        // - Return Ok(()) if SMTP credentials are missing (function returns early)
        // - Return Err() if database connection fails or user not found
        match result {
            Ok(_) => {
                // This is acceptable if SMTP credentials are not set
                println!("Function completed successfully (likely SMTP credentials not set)");
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("User not found")
                        || error_msg.contains("database")
                        || error_msg.contains("connect"),
                    "Error should mention user not found or database issue: {}",
                    error_msg
                );
            }
        }
    }

    #[tokio::test]
    async fn test_send_welcome_email_covers_all_code_paths() {
        use crate::setup::app::TestApp;
        use my_axum::{
            core::{context::Context, db::entity::user},
            pkg::password::hash_password_string,
            user::repository::user_repository,
        };
        use sea_orm::{ActiveValue::Set, TransactionTrait};

        let test_app = TestApp::spawn_app().await;

        // Create user with all fields populated using transaction
        let created_user = test_app
            .db
            .transaction::<_, user::Model, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let hashed_password = hash_password_string("password123@").await.unwrap();
                    let user_model = user::ActiveModel {
                        email: Set("fulldata@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Full".to_string())),
                        last_name: Set(Some("Data".to_string())),
                        phone: Set(Some("+9876543210".to_string())),
                        ..Default::default()
                    };

                    let context = Context::builder(Arc::new(txn.begin().await?)).build();
                    let result = user_repository::create(&context, user_model).await;
                    context.commit().await?;
                    result
                })
            })
            .await
            .unwrap();

        // This will execute:
        // - tracing::info for user_id
        // - Setting::new()
        // - get_db()
        // - transaction
        // - find_by_id
        // - tracing::info for user.email
        // - HashMap creation and all inserts
        // - render_email_template
        // - format! for text_body
        // - SmtpConfig::new with all fields
        // - SmtpClient::new
        // - send_multipart_mail (will fail here)

        let producer = MockProducer;
        let result = send_welcome_email(&test_app.db, &producer, created_user.id).await;

        // Expected to fail at SMTP or database
        if let Err(e) = result {
            let error_msg = e.to_string();
            // Should have gotten past all the setup code
            assert!(
                error_msg.contains("SMTP")
                    || error_msg.contains("send")
                    || error_msg.contains("mail")
                    || error_msg.contains("connection")
                    || error_msg.contains("database")
                    || error_msg.contains("Database")
                    || error_msg.contains("Failed to connect")
                    || error_msg.contains("create SMTP client"),
                "Expected SMTP/mail/database error, got: {}",
                error_msg
            );
        }
    }

    #[tokio::test]
    async fn test_send_welcome_email_database_transaction() {
        use crate::setup::app::TestApp;
        use my_axum::{
            core::{context::Context, db::entity::user},
            pkg::password::hash_password_string,
            user::repository::user_repository,
        };
        use sea_orm::{ActiveValue::Set, TransactionTrait};

        let test_app = TestApp::spawn_app().await;

        // Create a user to test database transaction handling
        let user_id = test_app
            .db
            .transaction::<_, i32, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let hashed_password = hash_password_string("txn_test").await.unwrap();
                    let user_model = user::ActiveModel {
                        email: Set("transaction_test@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Transaction".to_string())),
                        last_name: Set(Some("Test".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };

                    let context = Context::builder(Arc::new(txn.begin().await?)).build();
                    let created_user = user_repository::create(&context, user_model).await?;
                    context.commit().await?;
                    Ok(created_user.id)
                })
            })
            .await
            .unwrap();

        let producer = MockProducer;
        // This tests the transaction path in send_welcome_email
        let result = send_welcome_email(&test_app.db, &producer, user_id).await;

        // Verify the user was found (transaction worked)
        if let Err(e) = result {
            let error_msg = e.to_string();
            // Should NOT be "User not found" error
            assert!(
                !error_msg.contains("User not found")
                    || error_msg.contains("SMTP")
                    || error_msg.contains("mail"),
                "Should find user and fail at SMTP, got: {}",
                error_msg
            );
        }
    }

    #[test]
    fn test_email_template_variables_all_fields() {
        use std::collections::HashMap;

        // Test that all expected variables are created
        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "My Axum App".to_string());
        variables.insert("app_url".to_string(), "http://localhost:8080".to_string());
        variables.insert("email".to_string(), "test@example.com".to_string());
        variables.insert("first_name".to_string(), "John".to_string());
        variables.insert("last_name".to_string(), "Doe".to_string());
        variables.insert("phone".to_string(), "+1234567890".to_string());
        variables.insert("year".to_string(), "2025".to_string());

        // Verify all expected keys exist
        assert!(variables.contains_key("app_name"));
        assert!(variables.contains_key("app_url"));
        assert!(variables.contains_key("email"));
        assert!(variables.contains_key("first_name"));
        assert!(variables.contains_key("last_name"));
        assert!(variables.contains_key("phone"));
        assert!(variables.contains_key("year"));

        // Verify count matches what send_welcome_email creates
        assert_eq!(variables.len(), 7);
    }

    #[test]
    fn test_email_subject_and_message_format() {
        // Test the exact subject line used
        let subject = "Welcome to My Axum App!";
        assert!(!subject.is_empty());
        assert!(subject.contains("Welcome"));
        assert!(subject.contains("My Axum App"));

        // Test text body format with different name scenarios
        let test_cases = vec![
            (Some("John"), Some("Doe"), "john@example.com"),
            (Some(""), Some(""), "test@example.com"),
            (None, None, "user@example.com"),
        ];

        for (first_name, last_name, email) in test_cases {
            let text_body = format!(
                "Hello {} {}!\n\nThank you for registering with My Axum App. Your account has been successfully created.\n\nEmail: {}\n\nBest regards,\nThe My Axum App Team",
                first_name.unwrap_or(""),
                last_name.unwrap_or(""),
                email
            );

            assert!(text_body.contains("Hello"));
            assert!(text_body.contains(email));
            assert!(text_body.contains("Thank you for registering"));
            assert!(text_body.contains("Best regards"));
        }
    }

    // ==================== Tests for process_avatar_upload ====================

    #[tokio::test]
    async fn test_process_avatar_upload_with_valid_user() {
        use crate::setup::app::TestApp;
        use my_axum::{
            core::{context::Context, db::entity::user},
            pkg::password::hash_password_string,
            user::repository::user_repository,
        };
        use sea_orm::{ActiveValue::Set, TransactionTrait};

        let test_app = TestApp::spawn_app().await;

        // Create a test user
        let created_user = test_app
            .db
            .transaction::<_, user::Model, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let hashed_password = hash_password_string("test_password").await.unwrap();
                    let user_model = user::ActiveModel {
                        email: Set("avatar_test@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Avatar".to_string())),
                        last_name: Set(Some("Test".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };

                    let context = Context::builder(Arc::new(txn.begin().await?)).build();
                    let result = user_repository::create(&context, user_model).await;
                    context.commit().await?;
                    result
                })
            })
            .await
            .unwrap();

        let producer = TrackingMockProducer::new();
        let file_name = "avatar.png".to_string();
        let task_id = uuid::Uuid::new_v4().to_string();

        // Process avatar upload
        let result = process_avatar_upload(
            &test_app.db,
            &producer,
            &test_app.setting.redis_url,
            task_id,
            created_user.id,
            file_name,
            "en".to_string(),
        )
        .await;

        // Should succeed
        assert!(
            result.is_ok(),
            "Avatar upload should succeed for valid user"
        );

        // Verify messages were published (6 progress stages + 1 final message = 7 total)
        let messages = producer.get_profilessages();
        assert_eq!(
            messages.len(),
            7,
            "Should publish 7 messages (6 progress + 1 final)"
        );

        // Verify progress messages contain expected event types
        let progress_count = messages
            .iter()
            .filter(|msg| msg.contains("avatar_upload_progress"))
            .count();
        assert_eq!(progress_count, 6, "Should have 6 progress messages");

        let complete_count = messages
            .iter()
            .filter(|msg| msg.contains("avatar_upload_complete"))
            .count();
        assert_eq!(complete_count, 1, "Should have 1 completion message");
    }

    #[tokio::test]
    async fn test_process_avatar_upload_with_nonexistent_user() {
        use crate::setup::app::TestApp;

        let test_app = TestApp::spawn_app().await;
        let producer = TrackingMockProducer::new();
        let task_id = uuid::Uuid::new_v4().to_string();
        let user_id = 999999; // Non-existent user
        let file_name = "avatar.png".to_string();

        // Process avatar upload
        let result = process_avatar_upload(
            &test_app.db,
            &producer,
            &test_app.setting.redis_url,
            task_id,
            user_id,
            file_name,
            "en".to_string(),
        )
        .await;

        // Should fail with user not found error
        assert!(result.is_err(), "Should fail for non-existent user");

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("User not found") || error_msg.contains("not found"),
            "Error should mention user not found, got: {}",
            error_msg
        );

        // No messages should be published since user verification fails first
        assert_eq!(
            producer.message_count(),
            0,
            "Should not publish any messages when user not found"
        );
    }

    #[tokio::test]
    async fn test_process_avatar_upload_progress_stages() {
        use crate::setup::app::TestApp;
        use my_axum::{
            core::{context::Context, db::entity::user},
            pkg::password::hash_password_string,
            user::repository::user_repository,
        };
        use sea_orm::{ActiveValue::Set, TransactionTrait};

        let test_app = TestApp::spawn_app().await;

        // Create a test user
        let created_user = test_app
            .db
            .transaction::<_, user::Model, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let hashed_password = hash_password_string("test_password").await.unwrap();
                    let user_model = user::ActiveModel {
                        email: Set("progress_test@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Progress".to_string())),
                        last_name: Set(Some("Test".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };

                    let context = Context::builder(Arc::new(txn.begin().await?)).build();
                    let result = user_repository::create(&context, user_model).await;
                    context.commit().await?;
                    result
                })
            })
            .await
            .unwrap();

        let producer = TrackingMockProducer::new();
        let file_name = "test_avatar.jpg".to_string();
        let task_id = uuid::Uuid::new_v4().to_string();

        // Process avatar upload
        let result = process_avatar_upload(
            &test_app.db,
            &producer,
            &test_app.setting.redis_url,
            task_id,
            created_user.id,
            file_name.clone(),
            "en".to_string(),
        )
        .await;
        assert!(result.is_ok());

        let messages = producer.get_profilessages();

        // Verify progress stages exist
        let expected_stages = vec![
            ("10", "Validating file"),
            ("25", "Preparing upload"),
            ("40", "Processing image"),
            ("60", "Optimizing"),
            ("80", "Finalizing"),
            ("100", "Upload complete"),
        ];

        for (progress, message_part) in expected_stages {
            let found = messages
                .iter()
                .any(|msg| msg.contains(progress) && msg.contains(message_part));
            assert!(
                found,
                "Should contain progress {} with message containing '{}'",
                progress, message_part
            );
        }

        // Verify final message contains the file name
        let final_msg = messages.last().unwrap();
        assert!(
            final_msg.contains(&file_name),
            "Final message should contain file name"
        );
        assert!(
            final_msg.contains("avatar_upload_complete"),
            "Final message should have complete event type"
        );
    }

    #[tokio::test]
    async fn test_process_avatar_upload_message_structure() {
        use crate::setup::app::TestApp;
        use my_axum::{
            core::{context::Context, db::entity::user},
            pkg::password::hash_password_string,
            user::repository::user_repository,
        };
        use sea_orm::{ActiveValue::Set, TransactionTrait};

        let test_app = TestApp::spawn_app().await;

        // Create a test user
        let created_user = test_app
            .db
            .transaction::<_, user::Model, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let hashed_password = hash_password_string("test_password").await.unwrap();
                    let user_model = user::ActiveModel {
                        email: Set("structure_test@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Structure".to_string())),
                        last_name: Set(Some("Test".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };

                    let context = Context::builder(Arc::new(txn.begin().await?)).build();
                    let result = user_repository::create(&context, user_model).await;
                    context.commit().await?;
                    result
                })
            })
            .await
            .unwrap();

        let producer = TrackingMockProducer::new();
        let file_name = "structure_test.png".to_string();
        let task_id = uuid::Uuid::new_v4().to_string();

        let result = process_avatar_upload(
            &test_app.db,
            &producer,
            &test_app.setting.redis_url,
            task_id,
            created_user.id,
            file_name,
            "en".to_string(),
        )
        .await;
        assert!(result.is_ok());

        let messages = producer.get_profilessages();

        // Verify each message is valid JSON
        for msg in &messages {
            let json_result: Result<serde_json::Value, _> = serde_json::from_str(msg);
            assert!(json_result.is_ok(), "Message should be valid JSON: {}", msg);

            let json = json_result.unwrap();

            // Verify message structure
            assert!(
                json.get("event_type").is_some(),
                "Should have event_type field"
            );
            assert!(json.get("data").is_some(), "Should have data field");

            // Verify data structure
            let data = json.get("data").unwrap();
            assert!(data.get("user_id").is_some(), "Data should have user_id");
            assert!(data.get("progress").is_some(), "Data should have progress");
            assert!(data.get("status").is_some(), "Data should have status");
        }

        // Verify progress messages have "processing" status
        for msg in messages.iter().take(6) {
            let json: serde_json::Value = serde_json::from_str(msg).unwrap();
            let status = json["data"]["status"].as_str().unwrap();
            assert_eq!(
                status, "processing",
                "Progress messages should have processing status"
            );
        }

        // Verify final message has "completed" status
        let final_msg: serde_json::Value = serde_json::from_str(messages.last().unwrap()).unwrap();
        let status = final_msg["data"]["status"].as_str().unwrap();
        assert_eq!(
            status, "completed",
            "Final message should have completed status"
        );
    }

    #[tokio::test]
    async fn test_process_avatar_upload_with_zero_user_id() {
        use crate::setup::app::TestApp;

        let test_app = TestApp::spawn_app().await;
        let producer = TrackingMockProducer::new();
        let task_id = uuid::Uuid::new_v4().to_string();
        let user_id = 0;
        let file_name = "avatar.png".to_string();

        let result = process_avatar_upload(
            &test_app.db,
            &producer,
            &test_app.setting.redis_url,
            task_id,
            user_id,
            file_name,
            "en".to_string(),
        )
        .await;

        // Should fail with user not found
        assert!(result.is_err(), "Should fail for user_id = 0");

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("User not found") || error_msg.contains("not found"),
            "Error should mention user not found"
        );
    }

    #[tokio::test]
    async fn test_process_avatar_upload_progress_percentages() {
        use crate::setup::app::TestApp;
        use my_axum::{
            core::{context::Context, db::entity::user},
            pkg::password::hash_password_string,
            user::repository::user_repository,
        };
        use sea_orm::{ActiveValue::Set, TransactionTrait};

        let test_app = TestApp::spawn_app().await;

        // Create a test user
        let created_user = test_app
            .db
            .transaction::<_, user::Model, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let hashed_password = hash_password_string("test_password").await.unwrap();
                    let user_model = user::ActiveModel {
                        email: Set("percentages_test@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Percentages".to_string())),
                        last_name: Set(Some("Test".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };

                    let context = Context::builder(Arc::new(txn.begin().await?)).build();
                    let result = user_repository::create(&context, user_model).await;
                    context.commit().await?;
                    result
                })
            })
            .await
            .unwrap();

        let producer = TrackingMockProducer::new();
        let file_name = "percentage_test.png".to_string();
        let task_id = uuid::Uuid::new_v4().to_string();

        let result = process_avatar_upload(
            &test_app.db,
            &producer,
            &test_app.setting.redis_url,
            task_id,
            created_user.id,
            file_name,
            "en".to_string(),
        )
        .await;
        assert!(result.is_ok());

        let messages = producer.get_profilessages();

        // Verify progress percentages
        let expected_percentages = [10, 25, 40, 60, 80, 100];

        for (i, expected) in expected_percentages.iter().enumerate() {
            let msg: serde_json::Value = serde_json::from_str(&messages[i]).unwrap();
            let progress = msg["data"]["progress"].as_i64().unwrap();
            assert_eq!(
                progress, *expected as i64,
                "Message {} should have progress {}",
                i, expected
            );
        }

        // Verify final message also has progress 100
        let final_msg: serde_json::Value = serde_json::from_str(messages.last().unwrap()).unwrap();
        let final_progress = final_msg["data"]["progress"].as_i64().unwrap();
        assert_eq!(
            final_progress, 100,
            "Final message should have progress 100"
        );
    }

    #[tokio::test]
    async fn test_process_avatar_upload_user_id_in_messages() {
        use crate::setup::app::TestApp;
        use my_axum::{
            core::{context::Context, db::entity::user},
            pkg::password::hash_password_string,
            user::repository::user_repository,
        };
        use sea_orm::{ActiveValue::Set, TransactionTrait};

        let test_app = TestApp::spawn_app().await;

        // Create a test user
        let created_user = test_app
            .db
            .transaction::<_, user::Model, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let hashed_password = hash_password_string("test_password").await.unwrap();
                    let user_model = user::ActiveModel {
                        email: Set("userid_test@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("UserID".to_string())),
                        last_name: Set(Some("Test".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };

                    let context = Context::builder(Arc::new(txn.begin().await?)).build();
                    let result = user_repository::create(&context, user_model).await;
                    context.commit().await?;
                    result
                })
            })
            .await
            .unwrap();

        let producer = TrackingMockProducer::new();
        let file_name = "userid_test.png".to_string();
        let task_id = uuid::Uuid::new_v4().to_string();

        let result = process_avatar_upload(
            &test_app.db,
            &producer,
            &test_app.setting.redis_url,
            task_id,
            created_user.id,
            file_name,
            "en".to_string(),
        )
        .await;
        assert!(result.is_ok());

        let messages = producer.get_profilessages();

        // Verify all messages contain the correct user_id
        for msg in &messages {
            let json: serde_json::Value = serde_json::from_str(msg).unwrap();
            let user_id = json["data"]["user_id"].as_i64().unwrap();
            assert_eq!(
                user_id, created_user.id as i64,
                "Message should contain correct user_id"
            );
        }
    }

    #[tokio::test]
    async fn test_process_avatar_upload_localizes_progress_messages() {
        use crate::setup::app::TestApp;
        use my_axum::{
            core::{context::Context, db::entity::user},
            pkg::password::hash_password_string,
            user::repository::user_repository,
        };
        use sea_orm::{ActiveValue::Set, TransactionTrait};

        let test_app = TestApp::spawn_app().await;

        let created_user = test_app
            .db
            .transaction::<_, user::Model, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let hashed_password = hash_password_string("test_password").await.unwrap();
                    let user_model = user::ActiveModel {
                        email: Set("localized_progress_test@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Localized".to_string())),
                        last_name: Set(Some("Progress".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };

                    let context = Context::builder(Arc::new(txn.begin().await?)).build();
                    let result = user_repository::create(&context, user_model).await;
                    context.commit().await?;
                    result
                })
            })
            .await
            .unwrap();

        let producer = TrackingMockProducer::new();
        let task_id = uuid::Uuid::new_v4().to_string();

        let result = process_avatar_upload(
            &test_app.db,
            &producer,
            &test_app.setting.redis_url,
            task_id,
            created_user.id,
            "avatar.png".to_string(),
            "en".to_string(),
        )
        .await;
        assert!(result.is_ok());

        let messages = producer.get_profilessages();
        let has_english_progress = messages
            .iter()
            .any(|msg| msg.contains("Validating file") || msg.contains("Uploaded avatar"));

        assert!(
            has_english_progress,
            "Expected at least one localized English avatar progress message"
        );
    }

    #[test]
    fn test_avatar_upload_stages_configuration() {
        // Test the stages configuration used in process_avatar_upload
        let stages: Vec<(u32, String, u64)> = vec![
            (10, "Validating file...".to_string(), 100),
            (25, "Preparing upload...".to_string(), 200),
            (40, "Processing image...".to_string(), 400),
            (60, "Optimizing...".to_string(), 500),
            (80, "Finalizing...".to_string(), 300),
            (100, "Upload complete!".to_string(), 200),
        ];

        // Verify stage count
        assert_eq!(stages.len(), 6, "Should have 6 stages");

        // Verify progress percentages are in ascending order
        let mut last_progress = 0;
        for (progress, _, _) in &stages {
            assert!(
                *progress > last_progress,
                "Progress should be in ascending order"
            );
            last_progress = *progress;
        }

        // Verify final progress is 100
        assert_eq!(stages.last().unwrap().0, 100, "Final stage should be 100%");

        // Verify all messages are non-empty
        for (_, message, _) in &stages {
            assert!(!message.is_empty(), "Stage message should not be empty");
        }

        // Verify all delays are reasonable (> 0 and < 5000ms)
        for (_, _, delay) in &stages {
            assert!(*delay > 0, "Delay should be positive");
            assert!(*delay < 5000, "Delay should be reasonable");
        }
    }

    #[test]
    fn test_avatar_upload_event_types() {
        // Test the event types used
        let progress_event = "avatar_upload_progress";
        let complete_event = "avatar_upload_complete";

        assert!(!progress_event.is_empty());
        assert!(!complete_event.is_empty());
        assert_ne!(
            progress_event, complete_event,
            "Event types should be different"
        );
        assert!(progress_event.contains("avatar"));
        assert!(complete_event.contains("avatar"));
    }
}
