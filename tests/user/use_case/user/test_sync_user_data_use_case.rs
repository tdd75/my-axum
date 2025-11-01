#[cfg(test)]
mod sync_user_data_use_case_tests {
    use crate::setup::app::TestApp;
    use axum::http::StatusCode;
    use my_axum::user::entity::sea_orm_active_enums::UserRole;
    use my_axum::user::entity::user;
    use my_axum::user::use_case::user::sync_user_data_use_case;

    fn create_test_user() -> user::Model {
        user::Model {
            id: 1,
            email: "test@example.com".to_string(),
            password: "hashedpassword".to_string(),
            role: UserRole::User,
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
    async fn test_fetch_user_data_with_nonexistent_id() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            "999999".to_string().into(),
            "en".to_string(),
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::NOT_FOUND);
        assert!(!error.message.is_empty());
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
                "en".to_string(),
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

    #[tokio::test]
    async fn test_fetch_user_data_success_with_existing_user() {
        use my_axum::core::context::Context;
        use my_axum::user::dto::user_dto::UserCreateDTO;
        use my_axum::user::use_case::user::create_user_use_case;
        use sea_orm::TransactionTrait;
        use std::sync::Arc;

        let test_app = TestApp::spawn_app().await;

        // Create a real user inside a committed transaction
        let user_id = test_app
            .db
            .transaction::<_, i32, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context::builder(Arc::new(txn.begin().await?)).build();
                    let user_dto = UserCreateDTO {
                        email: "sync_success@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Sync".to_string()),
                        last_name: Some("Test".to_string()),
                        phone: None,
                    };
                    let created = create_user_use_case::execute(&context, user_dto)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    context.commit().await?;
                    Ok(created.data.id)
                })
            })
            .await
            .unwrap();

        let app_state = test_app.create_app_state();
        let test_user = create_test_user();

        let result = sync_user_data_use_case::fetch_user_data(
            app_state,
            test_user,
            user_id.to_string().into(),
            "en".to_string(),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.data.email, "sync_success@example.com");
        assert_eq!(response.data.first_name, Some("Sync".to_string()));
    }
}
