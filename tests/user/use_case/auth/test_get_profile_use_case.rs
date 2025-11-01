#[cfg(test)]
mod get_profile_use_case_tests {
    use crate::setup::app::TestApp;
    use my_axum::{
        core::{context::Context, db::entity::sea_orm_active_enums::UserRole},
        user::{
            dto::user_dto::UserCreateDTO,
            use_case::{auth::get_profile_use_case, user::create_user_use_case},
        },
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_profile_success() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let mut context = Context::builder(Arc::new(txn)).build();

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
            role: UserRole::User,
            first_name: created_user.first_name.clone(),
            last_name: created_user.last_name.clone(),
            phone: created_user.phone.clone(),
            created_at: created_user.created_at,
            updated_at: created_user.updated_at,
            created_user_id: None,
            updated_user_id: None,
        };

        // Create context with the user
        context.user = Some(user_model);

        // Execute get_profile
        let result = get_profile_use_case::execute(&context).await;

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
        let mut context = Context::builder(Arc::new(txn)).build();

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
            role: UserRole::User,
            first_name: created_user.first_name.clone(),
            last_name: created_user.last_name.clone(),
            phone: created_user.phone.clone(),
            created_at: created_user.created_at,
            updated_at: created_user.updated_at,
            created_user_id: None,
            updated_user_id: None,
        };

        // Create context with the user
        context.user = Some(user_model);

        let result = get_profile_use_case::execute(&context).await;

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
    async fn test_get_profile_preserves_user_model_data() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let mut context = Context::builder(Arc::new(txn)).build();

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
            role: UserRole::User,
            first_name: created_user.first_name.clone(),
            last_name: created_user.last_name.clone(),
            phone: created_user.phone.clone(),
            created_at: created_user.created_at,
            updated_at: created_user.updated_at,
            created_user_id: None,
            updated_user_id: None,
        };

        // Create context with the user
        context.user = Some(user_model);

        let result = get_profile_use_case::execute(&context).await;

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
        use serde_json;
        let serialized = serde_json::to_string(&user_data).unwrap();
        assert!(!serialized.contains("password"));
        assert!(!serialized.contains("secret_hashed_password"));
    }

    #[tokio::test]
    async fn test_get_profile_without_user_context() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        let result = get_profile_use_case::execute(&context).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, axum::http::StatusCode::UNAUTHORIZED);
    }
}
