#[cfg(test)]
mod refresh_token_use_case_tests {
    use crate::setup::app::TestApp;
    use axum::http::{HeaderMap, HeaderValue};
    use my_axum::{
        core::{context::Context, dto::error_dto::ErrorDTO},
        user::{
            dto::{
                auth_dto::{LoginDTO, RefreshTokenDTO},
                user_dto::UserCreateDTO,
            },
            repository::refresh_token_repository,
            use_case::{
                auth::{login_use_case, refresh_token_use_case},
                user::create_user_use_case,
            },
        },
    };

    #[tokio::test]
    async fn test_refresh_token_success_with_cookie() {
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
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Login to get tokens
        let login_dto = LoginDTO {
            email: user.email.clone(),
            password: "password123@".to_string(),
        };
        let headers = HeaderMap::new();
        let login_response = login_use_case::execute(&context, login_dto, headers)
            .await
            .unwrap();

        // Extract refresh token from login response headers
        let login_headers = login_response.headers.as_ref().unwrap();
        let refresh_token = extract_cookie_value(login_headers, "refresh_token")
            .expect("Should have refresh_token cookie");

        // Create headers with refresh token cookie
        let mut refresh_headers = HeaderMap::new();
        refresh_headers.insert(
            "cookie",
            HeaderValue::from_str(&format!("refresh_token={}", refresh_token)).unwrap(),
        );

        // Execute refresh token
        let dto = RefreshTokenDTO {
            refresh_token: None,
        }; // Token should come from cookie
        let result = refresh_token_use_case::execute(&context, dto, refresh_headers).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 200);

        // Check that new tokens are provided
        assert!(!response.data.access.is_empty());
        assert!(!response.data.refresh.is_empty());

        // Check that new cookies are set
        let response_headers = response.headers.as_ref().unwrap();
        let set_cookie_headers: Vec<&HeaderValue> =
            response_headers.get_all("set-cookie").iter().collect();

        assert!(set_cookie_headers.len() >= 2); // Should set both access and refresh tokens

        // Verify new tokens are different from original
        let new_refresh_token = extract_cookie_value(response_headers, "refresh_token")
            .expect("Should have new refresh_token cookie");
        assert_ne!(refresh_token, new_refresh_token);
    }

    #[tokio::test]
    async fn test_refresh_token_success_with_body() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user and login
        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let login_dto = LoginDTO {
            email: user.email.clone(),
            password: "password123@".to_string(),
        };
        let headers = HeaderMap::new();
        let login_response = login_use_case::execute(&context, login_dto, headers)
            .await
            .unwrap();

        let refresh_token = login_response.data.refresh.clone();

        // Execute refresh token with token in body
        let dto = RefreshTokenDTO {
            refresh_token: Some(refresh_token.clone()),
        };
        let headers = HeaderMap::new();
        let result = refresh_token_use_case::execute(&context, dto, headers).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 200);

        // Check that new tokens are provided and different
        assert!(!response.data.access.is_empty());
        assert!(!response.data.refresh.is_empty());
        assert_ne!(refresh_token, response.data.refresh);
    }

    #[tokio::test]
    async fn test_refresh_token_invalid_token() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Execute refresh with invalid token
        let dto = RefreshTokenDTO {
            refresh_token: Some("invalid_token_value".to_string()),
        };
        let headers = HeaderMap::new();
        let result = refresh_token_use_case::execute(&context, dto, headers).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 401);
    }

    #[tokio::test]
    async fn test_refresh_token_missing_token() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Execute refresh without token in body or cookie
        let dto = RefreshTokenDTO {
            refresh_token: None,
        };
        let headers = HeaderMap::new();
        let result = refresh_token_use_case::execute(&context, dto, headers).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 401);
    }

    #[tokio::test]
    async fn test_refresh_token_nonexistent_user() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create a JWT token with non-existent user ID
        use my_axum::config::setting::Setting;
        use my_axum::pkg::jwt::encode_token;

        let fake_user_id = 99999;
        let setting = Setting::new();
        let fake_token = encode_token(
            fake_user_id,
            chrono::Duration::minutes(15),
            &setting.jwt_secret,
        )
        .unwrap();

        let dto = RefreshTokenDTO {
            refresh_token: Some(fake_token),
        };
        let headers = HeaderMap::new();
        let result = refresh_token_use_case::execute(&context, dto, headers).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 401); // Invalid token, not user not found
    }

    #[tokio::test]
    async fn test_refresh_token_expired_token() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user and login
        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let login_dto = LoginDTO {
            email: user.email.clone(),
            password: "password123@".to_string(),
        };
        let headers = HeaderMap::new();
        let login_response = login_use_case::execute(&context, login_dto, headers)
            .await
            .unwrap();

        let refresh_token = login_response.data.refresh.clone();

        // Delete refresh token from database to simulate expiry
        let _ = refresh_token_repository::delete_by_token(&context, &refresh_token)
            .await
            .map_err(ErrorDTO::map_internal_error);

        // Try to use the refresh token
        let dto = RefreshTokenDTO {
            refresh_token: Some(refresh_token),
        };
        let headers = HeaderMap::new();
        let result = refresh_token_use_case::execute(&context, dto, headers).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 401);
    }

    #[tokio::test]
    async fn test_refresh_token_cookie_priority_over_body() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user and login
        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let login_dto = LoginDTO {
            email: user.email.clone(),
            password: "password123@".to_string(),
        };
        let headers = HeaderMap::new();
        let login_response = login_use_case::execute(&context, login_dto, headers)
            .await
            .unwrap();

        let login_headers = login_response.headers.as_ref().unwrap();
        let refresh_token_cookie = extract_cookie_value(login_headers, "refresh_token")
            .expect("Should have refresh_token cookie");
        let _refresh_token_body = login_response.data.refresh.clone(); // Not used but keeping for demonstration

        // Create headers with refresh token cookie
        let mut refresh_headers = HeaderMap::new();
        refresh_headers.insert(
            "cookie",
            HeaderValue::from_str(&format!("refresh_token={}", refresh_token_cookie)).unwrap(),
        );

        // Execute refresh with token in both cookie and body
        let dto = RefreshTokenDTO {
            refresh_token: Some("invalid_body_token".to_string()), // Invalid body token
        };
        let headers = refresh_headers;
        let result = refresh_token_use_case::execute(&context, dto, headers).await;

        // Should fail because body token has priority and it's invalid
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 401);
    }

    // Helper function to extract cookie value from headers
    fn extract_cookie_value(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
        headers
            .get_all("set-cookie")
            .iter()
            .find_map(|header_value| {
                let cookie_str = header_value.to_str().ok()?;
                if cookie_str.starts_with(&format!("{}=", cookie_name)) {
                    let value = cookie_str.split(';').next()?.split('=').nth(1)?.to_string();
                    Some(value)
                } else {
                    None
                }
            })
    }
}
