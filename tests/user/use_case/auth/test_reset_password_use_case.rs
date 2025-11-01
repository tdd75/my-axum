#[cfg(test)]
mod reset_password_use_case_tests {
    use crate::setup::app::TestApp;
    use chrono::{Duration, Utc};
    use my_axum::{
        core::{context::Context, db::entity::password_reset_token},
        pkg::password::verify_password,
        user::{
            dto::{auth_dto::ResetPasswordDTO, user_dto::UserCreateDTO},
            repository::{password_reset_repository, user_repository},
            use_case::{auth::reset_password_use_case, user::create_user_use_case},
        },
    };
    use sea_orm::Set;

    // Helper function to create password reset token
    async fn create_reset_token(
        context: &Context<'_>,
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
    async fn test_reset_password_success() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "old_password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Create expired OTP token
        let otp = "123456";
        create_reset_token(&context, created_user.id, otp).await;

        // Reset password with OTP
        let dto = ResetPasswordDTO {
            email: "test@example.com".to_string(),
            otp: otp.to_string(),
            new_password: "new_password456".to_string(),
        };

        let result = reset_password_use_case::execute(&context, dto).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 204);

        // Verify password was changed
        let updated_user = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        let password_matches = verify_password("new_password456", &updated_user.password).await;
        assert!(password_matches.is_ok());

        // Verify OTP was deleted
        let token = password_reset_repository::find_by_token(&context, otp)
            .await
            .unwrap();
        assert!(token.is_none());
    }

    #[tokio::test]
    async fn test_reset_password_invalid_otp() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "old_password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Create OTP token
        let otp = "123456";
        create_reset_token(&context, created_user.id, otp).await;

        // Try to reset with wrong OTP
        let dto = ResetPasswordDTO {
            email: "test@example.com".to_string(),
            otp: "wrong_otp".to_string(),
            new_password: "new_password456".to_string(),
        };

        let result = reset_password_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
        assert!(error.message.contains("Invalid email or OTP code"));
    }

    #[tokio::test]
    async fn test_reset_password_otp_belongs_to_different_user() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create first user
        let user1_dto = UserCreateDTO {
            email: "user1@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("User".to_string()),
            last_name: Some("One".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let user1 = create_user_use_case::execute(&context, user1_dto)
            .await
            .unwrap()
            .data;

        // Create second user
        let user2_dto = UserCreateDTO {
            email: "user2@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("User".to_string()),
            last_name: Some("Two".to_string()),
            phone: Some("0987654321".to_string()),
        };
        create_user_use_case::execute(&context, user2_dto)
            .await
            .unwrap();

        // Create OTP for user1
        let otp = "123456";
        create_reset_token(&context, user1.id, otp).await;

        // Try to use user1's OTP with user2's email
        let dto = ResetPasswordDTO {
            email: "user2@example.com".to_string(),
            otp: otp.to_string(),
            new_password: "new_password456".to_string(),
        };

        let result = reset_password_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
        assert!(error.message.contains("Invalid email or OTP code"));

        // Verify retry count was incremented
        let token = password_reset_repository::find_by_token(&context, otp)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(token.retry_count, 1);
    }

    #[tokio::test]
    async fn test_reset_password_expired_otp() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "old_password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Create expired OTP token (manually set expired time)
        let otp = "123456";
        let token = create_reset_token(&context, created_user.id, otp).await;

        // Manually update token to be expired
        use my_axum::core::db::entity::password_reset_token;
        use sea_orm::{Set, entity::*};

        let mut active_token: password_reset_token::ActiveModel = token.into();
        active_token.expires_at = Set((Utc::now() - Duration::minutes(1)).naive_utc());
        active_token.update(context.txn).await.unwrap();

        // Try to reset password with expired OTP
        let dto = ResetPasswordDTO {
            email: "test@example.com".to_string(),
            otp: otp.to_string(),
            new_password: "new_password456".to_string(),
        };

        let result = reset_password_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
        assert!(error.message.contains("expired"));

        // Verify token was deleted
        let token = password_reset_repository::find_by_token(&context, otp)
            .await
            .unwrap();
        assert!(token.is_none());
    }

    #[tokio::test]
    async fn test_reset_password_max_retry_attempts() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "old_password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Create OTP token
        let otp = "123456";
        let token = create_reset_token(&context, created_user.id, otp).await;

        // Manually set retry count to max (3)
        use my_axum::core::db::entity::password_reset_token;
        use sea_orm::{Set, entity::*};

        let mut active_token: password_reset_token::ActiveModel = token.into();
        active_token.retry_count = Set(3);
        active_token.update(context.txn).await.unwrap();

        // Try to reset password
        let dto = ResetPasswordDTO {
            email: "test@example.com".to_string(),
            otp: otp.to_string(),
            new_password: "new_password456".to_string(),
        };

        let result = reset_password_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
        assert!(error.message.contains("Maximum attempts"));

        // Verify token was deleted
        let token = password_reset_repository::find_by_token(&context, otp)
            .await
            .unwrap();
        assert!(token.is_none());
    }

    #[tokio::test]
    async fn test_reset_password_invalid_email_format() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = ResetPasswordDTO {
            email: "invalid-email".to_string(),
            otp: "123456".to_string(),
            new_password: "new_password456".to_string(),
        };

        let result = reset_password_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
    }

    #[tokio::test]
    async fn test_reset_password_empty_password() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "old_password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Create OTP token
        let otp = "123456";
        create_reset_token(&context, created_user.id, otp).await;

        // Try to reset with empty password
        let dto = ResetPasswordDTO {
            email: "test@example.com".to_string(),
            otp: otp.to_string(),
            new_password: "".to_string(), // Empty password
        };

        let result = reset_password_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
    }

    #[tokio::test]
    async fn test_reset_password_nonexistent_user() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = ResetPasswordDTO {
            email: "nonexistent@example.com".to_string(),
            otp: "123456".to_string(),
            new_password: "new_password456".to_string(),
        };

        let result = reset_password_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
        assert!(error.message.contains("Invalid email or OTP code"));
    }

    #[tokio::test]
    async fn test_reset_password_deletes_all_user_tokens() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "old_password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Create multiple OTP tokens
        let otp1 = "111111";
        let otp2 = "222222";
        create_reset_token(&context, created_user.id, otp1).await;
        create_reset_token(&context, created_user.id, otp2).await;

        // Reset password with first OTP
        let dto = ResetPasswordDTO {
            email: "test@example.com".to_string(),
            otp: otp1.to_string(),
            new_password: "new_password456".to_string(),
        };

        let result = reset_password_use_case::execute(&context, dto).await;
        assert!(result.is_ok());

        // Verify both tokens were deleted
        let token1 = password_reset_repository::find_by_token(&context, otp1)
            .await
            .unwrap();
        let token2 = password_reset_repository::find_by_token(&context, otp2)
            .await
            .unwrap();
        assert!(token1.is_none());
        assert!(token2.is_none());
    }
}
