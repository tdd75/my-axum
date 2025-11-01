#[cfg(test)]
mod get_profile_use_case_tests {
    use crate::setup::app::TestApp;
    use my_axum::{
        core::context::Context,
        user::{
            dto::user_dto::UserCreateDTO,
            use_case::{auth::get_profile_use_case, user::create_user_use_case},
        },
    };

    #[tokio::test]
    async fn test_get_profile_success() {
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
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Convert to user model (simulate what auth middleware would provide)
        use my_axum::core::db::entity::user;
        let user_model = user::Model {
            id: created_user.id,
            email: created_user.email.clone(),
            password: "hashed_password".to_string(), // Not exposed in response
            first_name: created_user.first_name.clone(),
            last_name: created_user.last_name.clone(),
            phone: created_user.phone.clone(),
            created_at: created_user.created_at,
            updated_at: created_user.updated_at,
            created_user_id: None,
            updated_user_id: None,
        };

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        // Execute get_profile
        let result = get_profile_use_case::execute(&context_with_user).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 200);

        // Verify returned data
        let user_data = response.data;
        assert_eq!(user_data.id, created_user.id);
        assert_eq!(user_data.email, created_user.email);
        assert_eq!(user_data.first_name, created_user.first_name);
        assert_eq!(user_data.last_name, created_user.last_name);
        assert_eq!(user_data.phone, created_user.phone);
        assert_eq!(user_data.created_at, created_user.created_at);
        assert_eq!(user_data.updated_at, created_user.updated_at);
    }

    #[tokio::test]
    async fn test_get_profile_with_minimal_user_data() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create user with minimal data
        let user_dto = UserCreateDTO {
            email: "minimal@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: None,
            last_name: None,
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        use my_axum::core::db::entity::user;
        let user_model = user::Model {
            id: created_user.id,
            email: created_user.email.clone(),
            password: "hashed_password".to_string(),
            first_name: created_user.first_name.clone(),
            last_name: created_user.last_name.clone(),
            phone: created_user.phone.clone(),
            created_at: created_user.created_at,
            updated_at: created_user.updated_at,
            created_user_id: None,
            updated_user_id: None,
        };

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        let result = get_profile_use_case::execute(&context_with_user).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 200);

        let user_data = response.data;
        assert_eq!(user_data.email, "minimal@example.com");
        assert_eq!(user_data.first_name, None);
        assert_eq!(user_data.last_name, None);
        assert_eq!(user_data.phone, None);
    }

    #[tokio::test]
    async fn test_get_profile_with_different_users() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create multiple users
        let user_dtos = vec![
            UserCreateDTO {
                email: "user1@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("User".to_string()),
                last_name: Some("One".to_string()),
                phone: Some("1111111111".to_string()),
            },
            UserCreateDTO {
                email: "user2@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("User".to_string()),
                last_name: Some("Two".to_string()),
                phone: Some("2222222222".to_string()),
            },
        ];

        for user_dto in user_dtos {
            let created_user = create_user_use_case::execute(&context, user_dto.clone())
                .await
                .unwrap()
                .data;

            use my_axum::core::db::entity::user;
            let user_model = user::Model {
                id: created_user.id,
                email: created_user.email.clone(),
                password: "hashed_password".to_string(),
                first_name: created_user.first_name.clone(),
                last_name: created_user.last_name.clone(),
                phone: created_user.phone.clone(),
                created_at: created_user.created_at,
                updated_at: created_user.updated_at,
                created_user_id: None,
                updated_user_id: None,
            };

            // Create context with the user
            let context_with_user = Context {
                txn: &txn,
                user: Some(user_model),
                producer: None,
            };

            let result = get_profile_use_case::execute(&context_with_user).await;

            assert!(result.is_ok());
            let response = result.unwrap();
            assert_eq!(response.status.as_u16(), 200);

            let user_data = response.data;
            assert_eq!(user_data.email, user_dto.email);
            assert_eq!(user_data.first_name, user_dto.first_name);
            assert_eq!(user_data.last_name, user_dto.last_name);
            assert_eq!(user_data.phone, user_dto.phone);
        }
    }

    #[tokio::test]
    async fn test_get_profile_response_structure() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "structure@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("Structure".to_string()),
            phone: Some("9999999999".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        use my_axum::core::db::entity::user;
        let user_model = user::Model {
            id: created_user.id,
            email: created_user.email.clone(),
            password: "hashed_password".to_string(),
            first_name: created_user.first_name.clone(),
            last_name: created_user.last_name.clone(),
            phone: created_user.phone.clone(),
            created_at: created_user.created_at,
            updated_at: created_user.updated_at,
            created_user_id: None,
            updated_user_id: None,
        };

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        let result = get_profile_use_case::execute(&context_with_user).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Test response structure
        assert_eq!(response.status.as_u16(), 200);
        assert!(response.data.id > 0);
        assert!(!response.data.email.is_empty());
        assert!(response.data.created_at.is_some());
        assert!(response.data.updated_at.is_some());

        // Ensure timestamps make sense
        if let (Some(created), Some(updated)) = (response.data.created_at, response.data.updated_at)
        {
            assert!(updated >= created);
        }
    }

    #[tokio::test]
    async fn test_get_profile_preserves_user_model_data() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create user with specific data
        let user_dto = UserCreateDTO {
            email: "preserve@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Preserve".to_string()),
            last_name: Some("Data".to_string()),
            phone: Some("5555555555".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        use my_axum::core::db::entity::user;
        let user_model = user::Model {
            id: created_user.id,
            email: created_user.email.clone(),
            password: "secret_hashed_password".to_string(), // This should NOT be in response
            first_name: created_user.first_name.clone(),
            last_name: created_user.last_name.clone(),
            phone: created_user.phone.clone(),
            created_at: created_user.created_at,
            updated_at: created_user.updated_at,
            created_user_id: None,
            updated_user_id: None,
        };

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        let result = get_profile_use_case::execute(&context_with_user).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let user_data = response.data;

        // Verify exact data preservation
        assert_eq!(user_data.id, created_user.id);
        assert_eq!(user_data.email, created_user.email);
        assert_eq!(user_data.first_name, created_user.first_name);
        assert_eq!(user_data.last_name, created_user.last_name);
        assert_eq!(user_data.phone, created_user.phone);
        assert_eq!(user_data.created_at, created_user.created_at);
        assert_eq!(user_data.updated_at, created_user.updated_at);

        // Make sure password is never exposed in UserDTO
        // (UserDTO should not have a password field)
        use serde_json;
        let serialized = serde_json::to_string(&user_data).unwrap();
        assert!(!serialized.contains("password"));
        assert!(!serialized.contains("secret_hashed_password"));
    }

    #[tokio::test]
    async fn test_get_profile_without_user_context() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None, // No user in context
            producer: None,
        };

        let result = get_profile_use_case::execute(&context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, axum::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_profile_error_handling() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        // Test with None user context multiple times to ensure consistency
        for _ in 0..3 {
            let context = Context {
                txn: &txn,
                user: None,
                producer: None,
            };

            let result = get_profile_use_case::execute(&context).await;
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert_eq!(error.status, axum::http::StatusCode::UNAUTHORIZED);
            assert!(
                error.message.contains("not_authenticated")
                    || error.message.contains("unauthorized")
            );
        }
    }

    #[tokio::test]
    async fn test_get_profile_context_user_cloning() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "clone@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Clone".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        use my_axum::core::db::entity::user;
        let user_model = user::Model {
            id: created_user.id,
            email: created_user.email.clone(),
            password: "hashed".to_string(),
            first_name: created_user.first_name.clone(),
            last_name: created_user.last_name.clone(),
            phone: None,
            created_at: created_user.created_at,
            updated_at: created_user.updated_at,
            created_user_id: None,
            updated_user_id: None,
        };

        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model.clone()),
            producer: None,
        };

        // Test that the function works with cloned context
        let result1 = get_profile_use_case::execute(&context_with_user).await;
        let result2 = get_profile_use_case::execute(&context_with_user).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Both results should be identical
        let user_data1 = result1.unwrap().data;
        let user_data2 = result2.unwrap().data;

        assert_eq!(user_data1.id, user_data2.id);
        assert_eq!(user_data1.email, user_data2.email);
    }

    #[tokio::test]
    async fn test_get_profile_user_service_integration() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create user with full data
        let user_dto = UserCreateDTO {
            email: "service@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Service".to_string()),
            last_name: Some("Test".to_string()),
            phone: Some("9876543210".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        use my_axum::core::db::entity::user;
        let user_model = user::Model {
            id: created_user.id,
            email: created_user.email.clone(),
            password: "hashed".to_string(),
            first_name: created_user.first_name.clone(),
            last_name: created_user.last_name.clone(),
            phone: created_user.phone.clone(),
            created_at: created_user.created_at,
            updated_at: created_user.updated_at,
            created_user_id: None,
            updated_user_id: None,
        };

        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        let result = get_profile_use_case::execute(&context_with_user).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, axum::http::StatusCode::OK);

        // Verify that user service conversion worked correctly
        let user_data = response.data;
        assert_eq!(user_data.email, "service@example.com");
        assert_eq!(user_data.first_name, Some("Service".to_string()));
        assert_eq!(user_data.last_name, Some("Test".to_string()));
        assert_eq!(user_data.phone, Some("9876543210".to_string()));
    }
}
