#[cfg(test)]
mod logout_use_case_tests {
    use crate::setup::app::TestApp;
    use axum::http::{HeaderMap, HeaderValue};
    use my_axum::{
        core::context::Context,
        user::{
            dto::{auth_dto::LoginDTO, user_dto::UserCreateDTO},
            repository::refresh_token_repository,
            use_case::{
                auth::{login_use_case, logout_use_case},
                user::create_user_use_case,
            },
        },
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_logout_verifies_refresh_token_deletion() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

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
        let refresh_token = extract_cookie_value(login_headers, "refresh_token")
            .expect("Should have refresh_token cookie");

        // Verify token exists in database before logout
        let token_before_logout = refresh_token_repository::find_by_token(&context, &refresh_token)
            .await
            .unwrap();
        assert!(token_before_logout.is_some());

        // Logout
        let mut logout_headers = HeaderMap::new();
        logout_headers.insert(
            "cookie",
            HeaderValue::from_str(&format!("refresh_token={}", refresh_token)).unwrap(),
        );

        let result = logout_use_case::execute(&context, logout_headers).await;
        assert!(result.is_ok());

        // Verify token is deleted from database after logout
        let token_after_logout = refresh_token_repository::find_by_token(&context, &refresh_token)
            .await
            .unwrap();
        assert!(token_after_logout.is_none()); // Should be deleted
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
