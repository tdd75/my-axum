#[cfg(test)]
mod auth_service_tests {
    use my_axum::config::setting::Setting;
    use my_axum::core::context::Context;
    use my_axum::pkg::jwt::decode_token;
    use my_axum::user::dto::user_dto::UserCreateDTO;
    use my_axum::user::service::auth_service::{generate_token_pair, get_current_user};
    use my_axum::user::use_case::user::create_user_use_case;

    use crate::setup::app::TestApp;
    use http::StatusCode;

    #[tokio::test]
    async fn test_generate_token_pair_success() {
        let user_id = 123;
        let result = generate_token_pair(user_id).await;

        assert!(result.is_ok());
        let (access_token, refresh_token) = result.unwrap();

        assert!(!access_token.is_empty());
        assert!(!refresh_token.is_empty());
        assert_ne!(access_token, refresh_token);
    }

    #[tokio::test]
    async fn test_generate_token_pair_tokens_are_valid() {
        let user_id = 456;
        let result = generate_token_pair(user_id).await;

        assert!(result.is_ok());
        let (access_token, refresh_token) = result.unwrap();

        let setting = Setting::new();

        // Verify access token can be decoded
        let access_claims = decode_token(&access_token, &setting.jwt_secret);
        assert!(access_claims.is_ok());
        assert_eq!(access_claims.unwrap().sub, user_id);

        // Verify refresh token can be decoded
        let refresh_claims = decode_token(&refresh_token, &setting.jwt_secret);
        assert!(refresh_claims.is_ok());
        assert_eq!(refresh_claims.unwrap().sub, user_id);
    }

    #[tokio::test]
    async fn test_generate_token_pair_different_users() {
        let user_id1 = 111;
        let user_id2 = 222;

        let result1 = generate_token_pair(user_id1).await;
        let result2 = generate_token_pair(user_id2).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let (access_token1, _) = result1.unwrap();
        let (access_token2, _) = result2.unwrap();

        // Tokens for different users should be different
        assert_ne!(access_token1, access_token2);
    }

    #[tokio::test]
    async fn test_generate_token_pair_multiple_calls() {
        let user_id = 789;

        let result1 = generate_token_pair(user_id).await;

        // Wait a bit to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let result2 = generate_token_pair(user_id).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let (access_token1, _) = result1.unwrap();
        let (access_token2, _) = result2.unwrap();

        // Multiple calls should generate different tokens (due to different timestamps)
        assert_ne!(access_token1, access_token2);
    }

    #[tokio::test]
    async fn test_get_current_user_invalid_token() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let token = "invalid_token_123";

        let result = get_current_user(&context, token).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::UNAUTHORIZED);
        assert!(error.message.contains("Invalid token"));
    }

    #[tokio::test]
    async fn test_get_current_user_token_for_nonexistent_user() {
        use my_axum::pkg::jwt::encode_token;

        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        // Create token for non-existent user ID
        let fake_user_id = 99999;
        let token = encode_token(
            fake_user_id,
            chrono::Duration::seconds(3600),
            &test_app.setting.jwt_secret,
        )
        .unwrap();

        let result = get_current_user(&context, &token).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::UNAUTHORIZED);
        assert!(error.message.contains("User not found"));
    }

    #[tokio::test]
    async fn test_get_current_user_success() {
        use my_axum::pkg::jwt::encode_token;

        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        // Create a test user
        let dto = UserCreateDTO {
            email: "auth_test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let user = create_user_use_case::execute(&context, dto)
            .await
            .unwrap()
            .data;
        let token = encode_token(
            user.id,
            chrono::Duration::seconds(3600),
            &test_app.setting.jwt_secret,
        )
        .unwrap();

        let result = get_current_user(&context, &token).await;
        assert!(result.is_ok());
        let found_user = result.unwrap();
        assert_eq!(found_user.id, user.id);
        assert_eq!(found_user.email, user.email);
    }
}
