#[cfg(test)]
mod login_use_case_tests {
    use crate::setup::app::TestApp;
    use axum::http::HeaderMap;
    use my_axum::{
        core::context::Context,
        user::{
            dto::{auth_dto::LoginDTO, user_dto::UserCreateDTO},
            use_case::{auth::login_use_case, user::create_user_use_case},
        },
    };

    #[tokio::test]
    async fn test_login_success() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
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

        let dto = LoginDTO {
            email: user.email.clone(),
            password: "password123@".to_string(), // Default test password
        };
        let headers = HeaderMap::new();

        let result = login_use_case::execute(&context, dto, headers).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 200);
        assert!(!response.data.access.is_empty());
        assert!(!response.data.refresh.is_empty());
        assert!(response.headers.is_some());
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let dto = LoginDTO {
            email: "nonexistent@example.com".to_string(),
            password: "password123@".to_string(),
        };
        let headers = HeaderMap::new();

        let result = login_use_case::execute(&context, dto, headers).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 401);
        assert_eq!(error.message, "Email has not been registered");
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
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

        let dto = LoginDTO {
            email: user.email.clone(),
            password: "wrong_password".to_string(),
        };
        let headers = HeaderMap::new();

        let result = login_use_case::execute(&context, dto, headers).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 401);
        assert_eq!(error.message, "Password is incorrect");
    }

    #[tokio::test]
    async fn test_login_empty_password() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
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

        let dto = LoginDTO {
            email: user.email.clone(),
            password: "".to_string(),
        };
        let headers = HeaderMap::new();

        let result = login_use_case::execute(&context, dto, headers).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 401);
        assert_eq!(error.message, "Password is incorrect");
    }

    #[tokio::test]
    async fn test_login_case_sensitive_email() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
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
        let _user = create_user_use_case::execute(&context, dto)
            .await
            .unwrap()
            .data;

        let dto = LoginDTO {
            email: "TEST@EXAMPLE.COM".to_string(),
            password: "password123@".to_string(),
        };

        let headers = HeaderMap::new();

        let result = login_use_case::execute(&context, dto, headers).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 401);
        assert_eq!(error.message, "Email has not been registered");
    }
}
