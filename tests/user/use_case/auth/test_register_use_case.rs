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
    use std::sync::Arc;

    #[tokio::test]
    async fn test_register_success_full_fields() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

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
    async fn test_register_duplicate_email() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

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
    async fn test_register_invalid_email_format() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

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
}
