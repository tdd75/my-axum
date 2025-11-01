#[cfg(test)]
mod forgot_password_use_case_tests {
    use crate::setup::app::TestApp;
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use my_axum::{
        core::{context::Context, db::entity::password_reset_token},
        pkg::messaging::MessageProducer,
        user::{
            dto::{auth_dto::ForgotPasswordDTO, user_dto::UserCreateDTO},
            repository::password_reset_repository,
            use_case::{auth::forgot_password_use_case, user::create_user_use_case},
        },
    };
    use sea_orm::Set;
    use std::sync::Arc;

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

    struct FailingProducer;

    #[async_trait]
    impl MessageProducer for FailingProducer {
        async fn publish_event_json(
            &self,
            _event_json: &str,
            _destination: Option<&str>,
        ) -> anyhow::Result<()> {
            Err(anyhow::anyhow!("Publish failed"))
        }
    }

    // Helper function to create password reset token
    async fn create_reset_token(
        context: &Context,
        user_id: i32,
        otp: &str,
    ) -> password_reset_token::Model {
        let expires_at = Utc::now() + Duration::minutes(15);
        let reset_token = password_reset_token::ActiveModel {
            user_id: Set(user_id),
            token: Set(otp.to_string()),
            retry_count: Set(0),
            expires_at: Set(expires_at.naive_utc()),
            ..Default::default()
        };
        password_reset_repository::create(context, reset_token)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_forgot_password_nonexistent_email() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        // Request password reset for non-existent email
        let dto = ForgotPasswordDTO {
            email: "nonexistent@example.com".to_string(),
        };

        let result = forgot_password_use_case::execute(&context, dto).await;

        // Should still return success to prevent email enumeration
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 204);
    }

    #[tokio::test]
    async fn test_forgot_password_invalid_email_format() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        // Request password reset with invalid email format
        let dto = ForgotPasswordDTO {
            email: "invalid-email".to_string(),
        };

        let result = forgot_password_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
        assert!(error.message.contains("email"));
    }

    #[tokio::test]
    async fn test_forgot_password_replaces_existing_token() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        // Create test user
        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Manually create first OTP token
        let otp1 = "111111";
        create_reset_token(&context, created_user.id, otp1).await;

        // Second password reset request (should replace first token)
        let dto2 = ForgotPasswordDTO {
            email: "test@example.com".to_string(),
        };
        let result2 = forgot_password_use_case::execute(&context, dto2).await;
        // Will fail due to no producer, but token should be replaced
        assert!(result2.is_err());

        // Verify first token was deleted and new one was created
        let token1 = password_reset_repository::find_by_token(&context, otp1)
            .await
            .unwrap();
        assert!(token1.is_none());

        // Clean up - new token exists
        password_reset_repository::delete_by_user_id(&context, created_user.id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_forgot_password_empty_email() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        // Request password reset with empty email
        let dto = ForgotPasswordDTO {
            email: "".to_string(),
        };

        let result = forgot_password_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
    }

    #[tokio::test]
    async fn test_forgot_password_multiple_requests_deletes_old_token() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        // Create test user
        let user_dto = UserCreateDTO {
            email: "multi@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: None,
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Create an old token manually
        let old_otp = "111111";
        let old_token = password_reset_token::ActiveModel {
            user_id: Set(created_user.id),
            token: Set(old_otp.to_string()),
            retry_count: Set(0),
            expires_at: Set((chrono::Utc::now() + Duration::minutes(15)).naive_utc()),
            ..Default::default()
        };
        password_reset_repository::create(&context, old_token)
            .await
            .unwrap();

        // Request password reset (should delete old token)
        let dto = ForgotPasswordDTO {
            email: "multi@example.com".to_string(),
        };

        let _result = forgot_password_use_case::execute(&context, dto).await;

        // Verify old token was deleted
        let old_token_result = password_reset_repository::find_by_token(&context, old_otp).await;
        assert!(old_token_result.is_ok());
        assert!(old_token_result.unwrap().is_none());

        // Clean up
        password_reset_repository::delete_by_user_id(&context, created_user.id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_forgot_password_success_with_producer() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(MockProducer));
        let context = Context::builder(Arc::new(txn)).producer(producer).build();

        // Create test user
        let user_dto = UserCreateDTO {
            email: "producer_test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap();

        // Request password reset
        let dto = ForgotPasswordDTO {
            email: "producer_test@example.com".to_string(),
        };

        let result = forgot_password_use_case::execute(&context, dto).await;

        // Should succeed with mock producer
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status.as_u16(), 204);
    }

    #[tokio::test]
    async fn test_forgot_password_success_user_without_first_name() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(MockProducer));
        let context = Context::builder(Arc::new(txn)).producer(producer).build();

        // Create user without first_name
        let user_dto = UserCreateDTO {
            email: "no_name_producer@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: None,
            last_name: None,
            phone: None,
        };
        create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap();

        let dto = ForgotPasswordDTO {
            email: "no_name_producer@example.com".to_string(),
        };

        let result = forgot_password_use_case::execute(&context, dto).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().status.as_u16(), 204);
    }

    #[tokio::test]
    async fn test_forgot_password_failing_producer() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(FailingProducer));
        let context = Context::builder(Arc::new(txn)).producer(producer).build();

        // Create test user
        let user_dto = UserCreateDTO {
            email: "fail_producer@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Fail".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };
        create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap();

        let dto = ForgotPasswordDTO {
            email: "fail_producer@example.com".to_string(),
        };

        let result = forgot_password_use_case::execute(&context, dto).await;

        // Should fail because producer fails to publish
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 500);
    }
}
