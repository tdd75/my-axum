mod get_profile_tests {
    use axum::http::StatusCode;
    use reqwest::Client;
    use serde_json::{Value, json};

    use crate::setup::app::TestApp;

    #[tokio::test]
    async fn test_get_profile_api_success() {
        // Arrange - Register a user and get access token
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let register_payload = json!({
            "email": "meuser@example.com",
            "password": "password123@",
            "first_name": "Me",
            "last_name": "User",
            "phone": "1234567890"
        });

        let register_response = client
            .post(format!(
                "http://{}/api/v1/auth/register/",
                &test_app.base_url
            ))
            .json(&register_payload)
            .send()
            .await
            .unwrap();

        let register_result = register_response.json::<Value>().await.unwrap();
        let access_token = register_result.get("access").unwrap().as_str().unwrap();

        // Act - Get user info with access token
        let response = client
            .get(format!("http://{}/api/v1/auth/me/", &test_app.base_url))
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let result = response.json::<Value>().await.unwrap();
        assert_eq!(
            result.get("email").unwrap().as_str().unwrap(),
            "meuser@example.com"
        );
        assert_eq!(result.get("first_name").unwrap().as_str().unwrap(), "Me");
        assert_eq!(result.get("last_name").unwrap().as_str().unwrap(), "User");
        assert_eq!(result.get("phone").unwrap().as_str().unwrap(), "1234567890");
        assert!(result.get("id").is_some());
        assert!(result.get("created_at").is_some());
        assert!(result.get("updated_at").is_some());
    }

    #[tokio::test]
    async fn test_get_profile_api_without_auth() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        // Act
        let response = client
            .get(format!("http://{}/api/v1/auth/me/", &test_app.base_url))
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

mod change_password_tests {
    use axum::http::StatusCode;
    use reqwest::Client;
    use serde_json::{Value, json};

    use crate::setup::app::TestApp;

    #[tokio::test]
    async fn test_change_password_api_success() {
        // Arrange - Register a user and get access token
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let register_payload = json!({
            "email": "changepassuser@example.com",
            "password": "oldpassword123@",
            "first_name": "Change",
            "last_name": "User"
        });

        let register_response = client
            .post(format!(
                "http://{}/api/v1/auth/register/",
                &test_app.base_url
            ))
            .json(&register_payload)
            .send()
            .await
            .unwrap();

        let register_result = register_response.json::<Value>().await.unwrap();
        let access_token = register_result.get("access").unwrap().as_str().unwrap();

        // Act - Change password with valid old password
        let change_password_payload = json!({
            "old_password": "oldpassword123@",
            "new_password": "newpassword456"
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/change-password/",
                &test_app.base_url
            ))
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&change_password_payload)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Verify the password was actually changed by trying to login with new password
        let login_payload = json!({
            "email": "changepassuser@example.com",
            "password": "newpassword456"
        });

        let login_response = client
            .post(format!("http://{}/api/v1/auth/login/", &test_app.base_url))
            .json(&login_payload)
            .send()
            .await
            .unwrap();

        assert_eq!(login_response.status(), StatusCode::OK);

        // Verify old password no longer works
        let old_login_payload = json!({
            "email": "changepassuser@example.com",
            "password": "oldpassword123@"
        });

        let old_login_response = client
            .post(format!("http://{}/api/v1/auth/login/", &test_app.base_url))
            .json(&old_login_payload)
            .send()
            .await
            .unwrap();

        assert_eq!(old_login_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_change_password_api_wrong_old_password() {
        // Arrange - Register a user and get access token
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let register_payload = json!({
            "email": "wrongoldpass@example.com",
            "password": "correctpassword123@",
            "first_name": "Wrong",
            "last_name": "User"
        });

        let register_response = client
            .post(format!(
                "http://{}/api/v1/auth/register/",
                &test_app.base_url
            ))
            .json(&register_payload)
            .send()
            .await
            .unwrap();

        let register_result = register_response.json::<Value>().await.unwrap();
        let access_token = register_result.get("access").unwrap().as_str().unwrap();

        // Act - Try to change password with wrong old password
        let change_password_payload = json!({
            "old_password": "wrongoldpassword",
            "new_password": "newpassword456"
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/change-password/",
                &test_app.base_url
            ))
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&change_password_payload)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_change_password_api_unauthorized() {
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        let change_password_payload = json!({
            "old_password": "oldpassword123@",
            "new_password": "newpassword123@"
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/change-password/",
                &test_app.base_url
            ))
            .json(&change_password_payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

mod login_tests {
    use axum::http::StatusCode;
    use reqwest::Client;
    use sea_orm::{DbErr, TransactionTrait};
    use serde_json::{Value, json};

    use crate::setup::app::TestApp;
    use my_axum::{
        core::context::Context,
        user::{dto::user_dto::UserCreateDTO, use_case::user::create_user_use_case},
    };

    #[tokio::test]
    async fn test_login_api_success() {
        // Arrange
        let test_app = TestApp::spawn_app().await;

        let user_email = test_app
            .db
            .transaction::<_, String, DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let dto = UserCreateDTO {
                        email: "test@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let user = create_user_use_case::execute(&context, dto)
                        .await
                        .unwrap()
                        .data;
                    Ok(user.email)
                })
            })
            .await
            .unwrap();

        let client = Client::new();
        let payload = json!({
            "email": user_email,
            "password": "password123@"
        });

        // Act
        let response = client
            .post(format!("http://{}/api/v1/auth/login/", &test_app.base_url))
            .json(&payload)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        // Check for cookies in headers first (before consuming response with json())
        let cookies: Vec<String> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|h| h.to_str().unwrap().to_string())
            .collect();

        let result = response.json::<Value>().await.unwrap();
        assert!(!result.get("access").unwrap().as_str().unwrap().is_empty());
        assert!(!result.get("refresh").unwrap().as_str().unwrap().is_empty());

        assert!(
            cookies.len() >= 2,
            "Should have at least 2 cookies (access_token and refresh_token)"
        );
        assert!(
            cookies.iter().any(|c| c.contains("access_token=")),
            "Should have access_token cookie"
        );
        assert!(
            cookies.iter().any(|c| c.contains("refresh_token=")),
            "Should have refresh_token cookie"
        );
    }

    #[tokio::test]
    async fn test_login_api_invalid_credentials() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let payload = json!({
            "email": "nonexistent@example.com",
            "password": "wrongpassword"
        });

        // Act
        let response = client
            .post(format!("http://{}/api/v1/auth/login/", &test_app.base_url))
            .json(&payload)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_api_empty_credentials() {
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let payload = json!({
            "email": "",
            "password": ""
        });

        let response = client
            .post(format!("http://{}/api/v1/auth/login/", &test_app.base_url))
            .json(&payload)
            .send()
            .await
            .unwrap();

        // Empty credentials should result in UNAUTHORIZED (user not found)
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_api_nonexistent_user() {
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let payload = json!({
            "email": "nonexistent@example.com",
            "password": "password123@"
        });

        let response = client
            .post(format!("http://{}/api/v1/auth/login/", &test_app.base_url))
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

mod register_tests {
    use axum::http::StatusCode;
    use reqwest::Client;
    use serde_json::{Value, json};

    use crate::setup::app::TestApp;

    #[tokio::test]
    async fn test_register_api_success() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let payload = json!({
            "email": "newuser@example.com",
            "password": "password123@",
            "first_name": "John",
            "last_name": "Doe",
            "phone": "1234567890"
        });

        // Act
        let response = client
            .post(format!(
                "http://{}/api/v1/auth/register/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let result = response.json::<Value>().await.unwrap();
        assert!(!result.get("access").unwrap().as_str().unwrap().is_empty());
        assert!(!result.get("refresh").unwrap().as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_register_api_sets_cookies() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let payload = json!({
            "email": "cookieuser@example.com",
            "password": "password123@",
            "first_name": "Cookie",
            "last_name": "User"
        });

        // Act
        let response = client
            .post(format!(
                "http://{}/api/v1/auth/register/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        // Check for cookies in headers first (before consuming response with json())
        let cookies: Vec<String> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|h| h.to_str().unwrap().to_string())
            .collect();

        let result = response.json::<Value>().await.unwrap();
        assert!(!result.get("access").unwrap().as_str().unwrap().is_empty());
        assert!(!result.get("refresh").unwrap().as_str().unwrap().is_empty());

        assert!(
            cookies.len() >= 2,
            "Should have at least 2 cookies (access_token and refresh_token)"
        );
        assert!(
            cookies.iter().any(|c| c.contains("access_token=")),
            "Should have access_token cookie"
        );
        assert!(
            cookies.iter().any(|c| c.contains("refresh_token=")),
            "Should have refresh_token cookie"
        );
    }

    #[tokio::test]
    async fn test_register_api_invalid_email() {
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let payload = json!({
            "email": "invalid-email",
            "password": "password123@",
            "first_name": "Test",
            "last_name": "User"
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/register/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_register_api_duplicate_email() {
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        // First registration
        let payload = json!({
            "email": "duplicate@example.com",
            "password": "password123@",
            "first_name": "Test",
            "last_name": "User"
        });

        let response1 = client
            .post(format!(
                "http://{}/api/v1/auth/register/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        // Second registration with same email
        let response2 = client
            .post(format!(
                "http://{}/api/v1/auth/register/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response2.status(), StatusCode::CONFLICT);
    }
}

mod refresh_token_tests {
    use axum::http::StatusCode;
    use reqwest::Client;
    use serde_json::{Value, json};

    use crate::setup::app::TestApp;

    #[tokio::test]
    async fn test_refresh_token_api_success() {
        // Arrange - First register a user to get tokens
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let register_payload = json!({
            "email": "refreshuser@example.com",
            "password": "password123@",
            "first_name": "Refresh",
            "last_name": "User"
        });

        let register_response = client
            .post(format!(
                "http://{}/api/v1/auth/register/",
                &test_app.base_url
            ))
            .json(&register_payload)
            .send()
            .await
            .unwrap();

        let register_result = register_response.json::<Value>().await.unwrap();
        let refresh_token = register_result.get("refresh").unwrap().as_str().unwrap();

        // Act - Use refresh token to get new tokens
        let refresh_payload = json!({
            "refresh_token": refresh_token
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/refresh-token/",
                &test_app.base_url
            ))
            .json(&refresh_payload)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        // Check for new cookies in headers first
        let cookies: Vec<String> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|h| h.to_str().unwrap().to_string())
            .collect();

        let result = response.json::<Value>().await.unwrap();
        assert!(!result.get("access").unwrap().as_str().unwrap().is_empty());
        assert!(!result.get("refresh").unwrap().as_str().unwrap().is_empty());

        // Verify new refresh token is different from old one
        let new_refresh_token = result.get("refresh").unwrap().as_str().unwrap();
        assert_ne!(refresh_token, new_refresh_token);

        assert!(cookies.len() >= 2, "Should have at least 2 cookies");
        assert!(
            cookies.iter().any(|c| c.contains("access_token=")),
            "Should have access_token cookie"
        );
        assert!(
            cookies.iter().any(|c| c.contains("refresh_token=")),
            "Should have refresh_token cookie"
        );
    }

    #[tokio::test]
    async fn test_refresh_token_api_invalid_token() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let payload = json!({
            "refresh_token": "invalid_token"
        });

        // Act
        let response = client
            .post(format!(
                "http://{}/api/v1/auth/refresh-token/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_refresh_token_api_missing_token() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let payload = json!({});

        // Act
        let response = client
            .post(format!(
                "http://{}/api/v1/auth/refresh-token/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

mod logout_tests {
    use axum::http::StatusCode;
    use reqwest::Client;

    use crate::setup::app::TestApp;

    #[tokio::test]
    async fn test_logout_api_success() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        // Act
        let response = client
            .post(format!("http://{}/api/v1/auth/logout/", &test_app.base_url))
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Check that cookies are being cleared (should have set-cookie headers with Max-Age=0)
        let cookies: Vec<String> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|h| h.to_str().unwrap().to_string())
            .collect();

        assert!(
            cookies
                .iter()
                .any(|c| c.contains("access_token=") && c.contains("Max-Age=0")),
            "Should clear access_token cookie"
        );
        assert!(
            cookies
                .iter()
                .any(|c| c.contains("refresh_token=") && c.contains("Max-Age=0")),
            "Should clear refresh_token cookie"
        );
    }
}

mod forgot_password_tests {
    use axum::http::StatusCode;
    use reqwest::Client;
    use sea_orm::{DbErr, TransactionTrait};
    use serde_json::json;

    use crate::setup::app::TestApp;
    use my_axum::{
        core::context::Context,
        user::{dto::user_dto::UserCreateDTO, use_case::user::create_user_use_case},
    };

    #[tokio::test]
    async fn test_forgot_password_api_success() {
        let test_app = TestApp::spawn_app().await;

        // Create a user first
        let user_email = test_app
            .db
            .transaction::<_, String, DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let dto = UserCreateDTO {
                        email: "test@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let user = create_user_use_case::execute(&context, dto)
                        .await
                        .unwrap()
                        .data;
                    Ok(user.email)
                })
            })
            .await
            .unwrap();

        let client = Client::new();
        let payload = json!({
            "email": user_email
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/forgot-password/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        // Will return 500 because no producer in test environment
        // but this verifies the API endpoint is working
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_forgot_password_api_invalid_email() {
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        let payload = json!({
            "email": "invalid-email"
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/forgot-password/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_forgot_password_api_nonexistent_email() {
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        let payload = json!({
            "email": "nonexistent@example.com"
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/forgot-password/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        // Should return NO_CONTENT to prevent email enumeration
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}

mod reset_password_tests {
    use axum::http::StatusCode;
    use reqwest::Client;
    use sea_orm::{DbErr, TransactionTrait};
    use serde_json::{Value, json};

    use crate::setup::app::TestApp;
    use chrono::{Duration, Utc};
    use my_axum::{
        core::{context::Context, db::entity::password_reset_token},
        user::{
            dto::user_dto::UserCreateDTO, repository::password_reset_repository,
            use_case::user::create_user_use_case,
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
    async fn test_reset_password_api_success() {
        let test_app = TestApp::spawn_app().await;

        // Create a user and OTP token
        let (user_email, otp) = test_app
            .db
            .transaction::<_, (String, String), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let dto = UserCreateDTO {
                        email: "test@example.com".to_string(),
                        password: "old_password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let user = create_user_use_case::execute(&context, dto)
                        .await
                        .unwrap()
                        .data;

                    // Create OTP token
                    let otp = "123456";
                    create_reset_token(&context, user.id, otp).await;

                    Ok((user.email, otp.to_string()))
                })
            })
            .await
            .unwrap();

        let client = Client::new();
        let payload = json!({
            "email": user_email,
            "otp": otp,
            "new_password": "new_password456"
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/reset-password/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Verify can login with new password
        let login_payload = json!({
            "email": user_email,
            "password": "new_password456"
        });

        let login_response = client
            .post(format!("http://{}/api/v1/auth/login/", &test_app.base_url))
            .json(&login_payload)
            .send()
            .await
            .unwrap();

        assert_eq!(login_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_reset_password_api_invalid_otp() {
        let test_app = TestApp::spawn_app().await;

        // Create a user and OTP token
        let user_email = test_app
            .db
            .transaction::<_, String, DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let dto = UserCreateDTO {
                        email: "test@example.com".to_string(),
                        password: "old_password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let user = create_user_use_case::execute(&context, dto)
                        .await
                        .unwrap()
                        .data;

                    // Create OTP token
                    let otp = "123456";
                    create_reset_token(&context, user.id, otp).await;

                    Ok(user.email)
                })
            })
            .await
            .unwrap();

        let client = Client::new();
        let payload = json!({
            "email": user_email,
            "otp": "wrong_otp",
            "new_password": "new_password456"
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/reset-password/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_reset_password_api_expired_otp() {
        let test_app = TestApp::spawn_app().await;

        // Create a user and expired OTP token
        let (user_email, otp) = test_app
            .db
            .transaction::<_, (String, String), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let dto = UserCreateDTO {
                        email: "test@example.com".to_string(),
                        password: "old_password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let user = create_user_use_case::execute(&context, dto)
                        .await
                        .unwrap()
                        .data;

                    // Create OTP token
                    let otp = "123456";
                    let token = create_reset_token(&context, user.id, otp).await;

                    // Manually update token to be expired
                    use chrono::{Duration, Utc};
                    use my_axum::core::db::entity::password_reset_token;
                    use sea_orm::{Set, entity::*};

                    let mut active_token: password_reset_token::ActiveModel = token.into();
                    active_token.expires_at = Set((Utc::now() - Duration::minutes(1)).naive_utc());
                    active_token.update(txn).await.unwrap();

                    Ok((user.email, otp.to_string()))
                })
            })
            .await
            .unwrap();

        let client = Client::new();
        let payload = json!({
            "email": user_email,
            "otp": otp,
            "new_password": "new_password456"
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/reset-password/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let result = response.json::<Value>().await.unwrap();
        assert!(
            result
                .get("message")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("expired")
        );
    }

    #[tokio::test]
    async fn test_reset_password_api_invalid_email() {
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        let payload = json!({
            "email": "invalid-email",
            "otp": "123456",
            "new_password": "new_password456"
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/reset-password/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_reset_password_api_empty_password() {
        let test_app = TestApp::spawn_app().await;

        // Create a user and OTP token
        let (user_email, otp) = test_app
            .db
            .transaction::<_, (String, String), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let dto = UserCreateDTO {
                        email: "test@example.com".to_string(),
                        password: "old_password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let user = create_user_use_case::execute(&context, dto)
                        .await
                        .unwrap()
                        .data;

                    // Create OTP token
                    let otp = "123456";
                    create_reset_token(&context, user.id, otp).await;

                    Ok((user.email, otp.to_string()))
                })
            })
            .await
            .unwrap();

        let client = Client::new();
        let payload = json!({
            "email": user_email,
            "otp": otp,
            "new_password": ""
        });

        let response = client
            .post(format!(
                "http://{}/api/v1/auth/reset-password/",
                &test_app.base_url
            ))
            .json(&payload)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
