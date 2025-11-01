#[cfg(test)]
mod update_profile_use_case_tests {
    use crate::setup::app::TestApp;
    use my_axum::{
        core::context::Context,
        user::entity::sea_orm_active_enums::UserRole,
        user::{
            dto::{auth_dto::UpdateProfileDTO, user_dto::UserCreateDTO},
            use_case::{auth::update_profile_use_case, user::create_user_use_case},
        },
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_profile_success() {
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

        // Convert to user model
        use my_axum::user::entity::user;
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

        // Update profile
        let update_dto = UpdateProfileDTO {
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            phone: Some("9876543210".to_string()),
        };
        let fields = vec![
            "first_name".to_string(),
            "last_name".to_string(),
            "phone".to_string(),
        ];

        let result = update_profile_use_case::execute(&context, update_dto, fields).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 200);

        // Verify updated data
        let profile = response.data;
        assert_eq!(profile.first_name, Some("Jane".to_string()));
        assert_eq!(profile.last_name, Some("Smith".to_string()));
        assert_eq!(profile.phone, Some("9876543210".to_string()));
        assert_eq!(profile.email, created_user.email);
    }

    #[tokio::test]
    async fn test_update_profile_without_user_context() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        let update_dto = UpdateProfileDTO {
            first_name: Some("Test".to_string()),
            last_name: None,
            phone: None,
        };
        let fields = vec!["first_name".to_string()];

        let result = update_profile_use_case::execute(&context, update_dto, fields).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, axum::http::StatusCode::UNAUTHORIZED);
    }
}
