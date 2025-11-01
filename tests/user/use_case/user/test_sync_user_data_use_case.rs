#[cfg(test)]
mod sync_user_data_use_case_tests {
    use crate::setup::app::TestApp;
    use axum::http::StatusCode;
    use my_axum::core::db::entity::user;
    use my_axum::user::use_case::user::sync_user_data_use_case;

    fn create_test_user() -> user::Model {
        user::Model {
            id: 1,
            email: "test@example.com".to_string(),
            password: "hashedpassword".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
            created_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
            created_user_id: None,
            updated_user_id: None,
        }
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_invalid_id_format() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "not_a_number".to_string().into(),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Invalid user ID format");
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_nonexistent_id() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "999999".to_string().into(),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::NOT_FOUND);
        assert_eq!(error.message, "User not found");
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_empty_string() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result =
            sync_user_data_use_case::fetch_user_data(app_state, test_user, "".to_string().into())
                .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_special_characters() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "abc@#$".to_string().into(),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_negative_id() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result =
            sync_user_data_use_case::fetch_user_data(app_state, test_user, "-5".to_string().into())
                .await;

        // Negative IDs should parse correctly but not be found
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_zero_id() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result =
            sync_user_data_use_case::fetch_user_data(app_state, test_user, "0".to_string().into())
                .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_large_id() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "2147483647".to_string().into(), // Max i32
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_whitespace() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "  123  ".to_string().into(),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_decimal() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "123.45".to_string().into(),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_hex_format() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "0x123".to_string().into(),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_valid_positive_id() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result =
            sync_user_data_use_case::fetch_user_data(app_state, test_user, "1".to_string().into())
                .await;

        // This should succeed or fail based on whether user exists
        // Either way, it should parse the ID correctly
        match result {
            Ok(response) => {
                assert_eq!(response.status, StatusCode::OK);
            }
            Err(error) => {
                assert_eq!(error.status, StatusCode::NOT_FOUND);
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_user_data_error_message_format() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "invalid".to_string().into(),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(!error.message.is_empty());
        assert!(error.message.contains("Invalid"));
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_unicode_characters() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "一二三".to_string().into(),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_very_long_string() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let long_string = "1".repeat(1000);
        let result =
            sync_user_data_use_case::fetch_user_data(app_state, test_user, long_string.into())
                .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_fetch_user_data_with_multiple_numbers() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "123 456".to_string().into(),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_fetch_user_data_boundary_values() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        let test_cases = vec![
            ("1", false),           // Valid format
            ("100", false),         // Valid format
            ("-1", false),          // Valid format but likely not found
            ("0", false),           // Valid format
            ("", true),             // Invalid - empty
            ("abc", true),          // Invalid - letters
            ("12.5", true),         // Invalid - decimal
            ("999999999999", true), // Invalid - overflow
        ];

        for (input, should_be_bad_request) in test_cases {
            let test_user = create_test_user();
            let result = sync_user_data_use_case::fetch_user_data(
                app_state.clone(),
                test_user,
                input.to_string().into(),
            )
            .await;

            assert!(result.is_err());
            let error = result.unwrap_err();

            if should_be_bad_request {
                assert_eq!(
                    error.status,
                    StatusCode::BAD_REQUEST,
                    "Input '{}' should return BAD_REQUEST",
                    input
                );
            }
        }
    }
}
