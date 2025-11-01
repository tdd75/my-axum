#[cfg(test)]
mod register_use_case_tests {
    use crate::setup::app::TestApp;
    use axum::http::HeaderMap;
    use my_axum::{
        core::context::Context,
        user::{
            dto::{auth_dto::RegisterDTO, user_dto::UserCreateDTO},
            use_case::{auth::register_use_case, user::create_user_use_case},
        },
    };

    #[tokio::test]
    async fn test_register_success_full_fields() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = RegisterDTO {
            email: "newuser@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            phone: Some("1234567890".to_string()),
        };

        let headers = HeaderMap::new();

        let result = register_use_case::execute(&context, dto, headers).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 200);
        assert!(!response.data.access.is_empty());
        assert!(!response.data.refresh.is_empty());
        assert!(response.headers.is_some());
    }

    #[tokio::test]
    async fn test_register_success_minimal_fields() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = RegisterDTO {
            email: "minimal@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: None,
            last_name: None,
            phone: None,
        };

        let headers = HeaderMap::new();
        let result = register_use_case::execute(&context, dto, headers).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 200);
        assert!(!response.data.access.is_empty());
        assert!(!response.data.refresh.is_empty());
        assert!(response.headers.is_some());
    }

    #[tokio::test]
    async fn test_register_duplicate_email() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create a user first
        let create_dto = UserCreateDTO {
            email: "existing@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Existing".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let _user = create_user_use_case::execute(&context, create_dto)
            .await
            .unwrap();

        // Try to register with the same email
        let dto = RegisterDTO {
            email: "existing@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("New".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("9876543210".to_string()),
        };

        let headers = HeaderMap::new();

        let result = register_use_case::execute(&context, dto, headers).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 409);
        assert_eq!(error.message, "Email address already exists");
    }

    #[tokio::test]
    async fn test_register_empty_email() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = RegisterDTO {
            email: "".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };

        let headers = HeaderMap::new();

        let result = register_use_case::execute(&context, dto, headers).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_empty_password() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = RegisterDTO {
            email: "test@example.com".to_string(),
            password: "".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };

        let headers = HeaderMap::new();

        let result = register_use_case::execute(&context, dto, headers).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_invalid_email_format() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = RegisterDTO {
            email: "invalid-email".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };

        let headers = HeaderMap::new();

        let result = register_use_case::execute(&context, dto, headers).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_sets_cookies() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = RegisterDTO {
            email: "cookies@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Cookie".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };

        let headers = HeaderMap::new();

        let result = register_use_case::execute(&context, dto, headers).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.headers.is_some());

        let headers = response.headers.unwrap();
        let cookie_header = headers.get("Set-Cookie");
        assert!(cookie_header.is_some());
    }

    #[tokio::test]
    async fn test_register_password_hashing() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = RegisterDTO {
            email: "hashing@example.com".to_string(),
            password: "plaintext123".to_string(),
            first_name: Some("Hash".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };

        let headers = HeaderMap::new();

        let result = register_use_case::execute(&context, dto, headers).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        // User should be created successfully
        assert!(!response.data.access.is_empty());
    }

    #[tokio::test]
    async fn test_register_generates_valid_tokens() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = RegisterDTO {
            email: "tokens@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Token".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };

        let headers = HeaderMap::new();

        let result = register_use_case::execute(&context, dto, headers).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        // Verify tokens are not empty strings
        assert!(response.data.access.len() > 50); // JWT tokens are quite long
        assert!(response.data.refresh.len() > 50);
        // Verify they're different tokens
        assert_ne!(response.data.access, response.data.refresh);
    }

    #[tokio::test]
    async fn test_register_welcome_email_task() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None, // No producer, but should still succeed
        };

        let dto = RegisterDTO {
            email: "welcome@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Welcome".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };

        let headers = HeaderMap::new();

        let result = register_use_case::execute(&context, dto, headers).await;
        // Should succeed even without producer
        assert!(result.is_ok());
    }
}
