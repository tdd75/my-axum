#[cfg(test)]
mod update_profile_use_case_tests {
    use crate::setup::app::TestApp;
    use my_axum::{
        core::context::Context,
        user::{
            dto::{auth_dto::UpdateProfileDTO, user_dto::UserCreateDTO},
            use_case::{auth::update_profile_use_case, user::create_user_use_case},
        },
    };

    #[tokio::test]
    async fn test_update_profile_success() {
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

        // Convert to user model
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

        let result = update_profile_use_case::execute(&context_with_user, update_dto, fields).await;

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
    async fn test_update_profile_partial_fields() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "partial@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Original".to_string()),
            last_name: Some("Name".to_string()),
            phone: Some("1111111111".to_string()),
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

        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        // Update only first_name
        let update_dto = UpdateProfileDTO {
            first_name: Some("Updated".to_string()),
            last_name: Some("Name".to_string()), // Provided but not in fields
            phone: Some("1111111111".to_string()), // Provided but not in fields
        };
        let fields = vec!["first_name".to_string()];

        let result = update_profile_use_case::execute(&context_with_user, update_dto, fields).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let profile = response.data;

        // Only first_name should be updated
        assert_eq!(profile.first_name, Some("Updated".to_string()));
        // Other fields should remain unchanged
        assert_eq!(profile.last_name, Some("Name".to_string()));
        assert_eq!(profile.phone, Some("1111111111".to_string()));
    }

    #[tokio::test]
    async fn test_update_profile_clear_optional_fields() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user with all fields
        let user_dto = UserCreateDTO {
            email: "clear@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("First".to_string()),
            last_name: Some("Last".to_string()),
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
            password: "hashed_password".to_string(),
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

        // Clear optional fields by setting to None
        let update_dto = UpdateProfileDTO {
            first_name: None,
            last_name: None,
            phone: None,
        };
        let fields = vec![
            "first_name".to_string(),
            "last_name".to_string(),
            "phone".to_string(),
        ];

        let result = update_profile_use_case::execute(&context_with_user, update_dto, fields).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let profile = response.data;

        // All fields should be cleared
        assert_eq!(profile.first_name, None);
        assert_eq!(profile.last_name, None);
        assert_eq!(profile.phone, None);
        assert_eq!(profile.email, created_user.email);
    }

    #[tokio::test]
    async fn test_update_profile_empty_fields_list() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "empty@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Original".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("3333333333".to_string()),
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

        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        // Provide update data but no fields to update
        let update_dto = UpdateProfileDTO {
            first_name: Some("Changed".to_string()),
            last_name: Some("Data".to_string()),
            phone: Some("9999999999".to_string()),
        };
        let fields = vec![]; // Empty fields list

        let result = update_profile_use_case::execute(&context_with_user, update_dto, fields).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        let profile = response.data;

        // Nothing should be updated
        assert_eq!(profile.first_name, Some("Original".to_string()));
        assert_eq!(profile.last_name, Some("User".to_string()));
        assert_eq!(profile.phone, Some("3333333333".to_string()));
    }

    #[tokio::test]
    async fn test_update_profile_without_user_context() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None, // No user in context
            producer: None,
        };

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

    #[tokio::test]
    async fn test_update_profile_preserves_email() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "preserve@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
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
            password: "hashed_password".to_string(),
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

        // Update profile
        let update_dto = UpdateProfileDTO {
            first_name: Some("Updated".to_string()),
            last_name: Some("Name".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let fields = vec![
            "first_name".to_string(),
            "last_name".to_string(),
            "phone".to_string(),
        ];

        let result = update_profile_use_case::execute(&context_with_user, update_dto, fields).await;

        assert!(result.is_ok());
        let profile = result.unwrap().data;

        // Email should remain unchanged (not updatable via this endpoint)
        assert_eq!(profile.email, "preserve@example.com");
    }

    #[tokio::test]
    async fn test_update_profile_multiple_times() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "multiple@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Initial".to_string()),
            last_name: Some("Name".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        use my_axum::core::db::entity::user;
        let mut user_model = user::Model {
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

        // First update
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model.clone()),
            producer: None,
        };

        let update_dto = UpdateProfileDTO {
            first_name: Some("First Update".to_string()),
            last_name: None,
            phone: None,
        };
        let fields = vec!["first_name".to_string()];

        let result = update_profile_use_case::execute(&context_with_user, update_dto, fields).await;
        assert!(result.is_ok());
        let profile = result.unwrap().data;
        assert_eq!(profile.first_name, Some("First Update".to_string()));

        // Update user_model for second update
        user_model.first_name = Some("First Update".to_string());

        // Second update
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        let update_dto = UpdateProfileDTO {
            first_name: None,
            last_name: Some("Second Update".to_string()),
            phone: Some("1112223333".to_string()),
        };
        let fields = vec!["last_name".to_string(), "phone".to_string()];

        let result = update_profile_use_case::execute(&context_with_user, update_dto, fields).await;
        assert!(result.is_ok());
        let profile = result.unwrap().data;
        assert_eq!(profile.first_name, Some("First Update".to_string()));
        assert_eq!(profile.last_name, Some("Second Update".to_string()));
        assert_eq!(profile.phone, Some("1112223333".to_string()));
    }

    #[tokio::test]
    async fn test_update_profile_with_long_values() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "long@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Short".to_string()),
            last_name: Some("Name".to_string()),
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

        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        // Update with long values
        let long_first_name = "A".repeat(100);
        let long_last_name = "B".repeat(100);
        let long_phone = "1234567890123456789012345678901234567890".to_string();

        let update_dto = UpdateProfileDTO {
            first_name: Some(long_first_name.clone()),
            last_name: Some(long_last_name.clone()),
            phone: Some(long_phone.clone()),
        };
        let fields = vec![
            "first_name".to_string(),
            "last_name".to_string(),
            "phone".to_string(),
        ];

        let result = update_profile_use_case::execute(&context_with_user, update_dto, fields).await;

        assert!(result.is_ok());
        let profile = result.unwrap().data;
        assert_eq!(profile.first_name, Some(long_first_name));
        assert_eq!(profile.last_name, Some(long_last_name));
        assert_eq!(profile.phone, Some(long_phone));
    }

    #[tokio::test]
    async fn test_update_profile_response_structure() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create test user
        let user_dto = UserCreateDTO {
            email: "structure@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
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
            password: "hashed_password".to_string(),
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

        let update_dto = UpdateProfileDTO {
            first_name: Some("Updated".to_string()),
            last_name: None,
            phone: None,
        };
        let fields = vec!["first_name".to_string()];

        let result = update_profile_use_case::execute(&context_with_user, update_dto, fields).await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Verify response structure
        assert_eq!(response.status, axum::http::StatusCode::OK);
        assert!(response.data.id > 0);
        assert!(!response.data.email.is_empty());
        assert!(response.data.created_at.is_some());
        assert!(response.data.updated_at.is_some());

        // Verify password is not exposed
        use serde_json;
        let serialized = serde_json::to_string(&response.data).unwrap();
        assert!(!serialized.contains("password"));
    }
}
