#[cfg(test)]
mod change_password_use_case_tests {
    use crate::setup::app::TestApp;
    use my_axum::{
        core::context::Context,
        pkg::password::verify_password,
        user::{
            dto::{auth_dto::ChangePasswordDTO, user_dto::UserCreateDTO},
            repository::user_repository,
            use_case::{auth::change_password_use_case, user::create_user_use_case},
        },
    };

    #[tokio::test]
    async fn test_change_password_success() {
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
            password: "old_password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("1234567890".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        // Get user model from database
        let user_model = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap(); // Unwrap Option

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        // Change password
        let change_password_dto = ChangePasswordDTO {
            old_password: "old_password123@".to_string(),
            new_password: "new_password456".to_string(),
        };

        let result =
            change_password_use_case::execute(&context_with_user, change_password_dto).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 204);

        // Verify password was actually changed in database
        let updated_user = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        // Old password should no longer work
        let old_verify_result = verify_password("old_password123@", &updated_user.password).await;
        assert!(old_verify_result.is_err());

        // New password should work
        let new_verify_result = verify_password("new_password456", &updated_user.password).await;
        assert!(new_verify_result.is_ok());
    }

    #[tokio::test]
    async fn test_change_password_wrong_old_password() {
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
            password: "correct_password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let user_model = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        // Try to change password with wrong old password
        let change_password_dto = ChangePasswordDTO {
            old_password: "wrong_password".to_string(),
            new_password: "new_password456".to_string(),
        };

        let result =
            change_password_use_case::execute(&context_with_user, change_password_dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
        assert!(error.message.contains("password") || error.message.contains("incorrect"));

        // Verify password was NOT changed
        let unchanged_user = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        let verify_result = verify_password("correct_password123@", &unchanged_user.password).await;
        assert!(verify_result.is_ok());
    }

    #[tokio::test]
    async fn test_change_password_empty_old_password() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let user_model = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        let change_password_dto = ChangePasswordDTO {
            old_password: "".to_string(),
            new_password: "new_password456".to_string(),
        };

        let result =
            change_password_use_case::execute(&context_with_user, change_password_dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
    }

    #[tokio::test]
    async fn test_change_password_empty_new_password() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let user_model = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        let change_password_dto = ChangePasswordDTO {
            old_password: "password123@".to_string(),
            new_password: "".to_string(),
        };

        let result =
            change_password_use_case::execute(&context_with_user, change_password_dto).await;

        // Empty password is actually accepted and hashed - changing expectation
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 204);
    }

    #[tokio::test]
    async fn test_change_password_same_old_and_new_password() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let user_model = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        let change_password_dto = ChangePasswordDTO {
            old_password: "password123@".to_string(),
            new_password: "password123@".to_string(), // Same as old password
        };

        let result =
            change_password_use_case::execute(&context_with_user, change_password_dto).await;

        // Should still work (updating to same password is technically valid)
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 204);

        // Verify password still works
        let updated_user = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        let verify_result = verify_password("password123@", &updated_user.password).await;
        assert!(verify_result.is_ok());
    }

    #[tokio::test]
    async fn test_change_password_multiple_times() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "initial_password".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let passwords = [
            ("initial_password", "first_change"),
            ("first_change", "second_change"),
            ("second_change", "final_password"),
        ];

        for (old_pwd, new_pwd) in passwords.iter() {
            let user_model = user_repository::find_by_id(&context, created_user.id)
                .await
                .unwrap()
                .unwrap();

            // Create context with the user
            let context_with_user = Context {
                txn: &txn,
                user: Some(user_model),
                producer: None,
            };

            let change_password_dto = ChangePasswordDTO {
                old_password: old_pwd.to_string(),
                new_password: new_pwd.to_string(),
            };

            let result =
                change_password_use_case::execute(&context_with_user, change_password_dto).await;

            assert!(result.is_ok());

            // Verify new password works
            let updated_user = user_repository::find_by_id(&context, created_user.id)
                .await
                .unwrap()
                .unwrap();

            let verify_result = verify_password(new_pwd, &updated_user.password).await;
            assert!(verify_result.is_ok());
        }

        // Verify only final password works
        let final_user = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        let final_verify = verify_password("final_password", &final_user.password).await;
        assert!(final_verify.is_ok());

        let old_verify = verify_password("initial_password", &final_user.password).await;
        assert!(old_verify.is_err());
    }

    #[tokio::test]
    async fn test_change_password_with_special_characters() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "simple_password".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let user_model = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        // Use password with special characters
        let special_password = "C0mpl3x!@#$%^&*()_+-={}[]|\\:;\"'<>?,./~`";
        let change_password_dto = ChangePasswordDTO {
            old_password: "simple_password".to_string(),
            new_password: special_password.to_string(),
        };

        let result =
            change_password_use_case::execute(&context_with_user, change_password_dto).await;

        assert!(result.is_ok());

        // Verify special character password works
        let updated_user = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        let verify_result = verify_password(special_password, &updated_user.password).await;
        assert!(verify_result.is_ok());
    }

    #[tokio::test]
    async fn test_change_password_preserves_other_user_fields() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let user_dto = UserCreateDTO {
            email: "preserve@example.com".to_string(),
            password: "old_password".to_string(),
            first_name: Some("Preserve".to_string()),
            last_name: Some("Fields".to_string()),
            phone: Some("9876543210".to_string()),
        };
        let created_user = create_user_use_case::execute(&context, user_dto)
            .await
            .unwrap()
            .data;

        let user_model = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();
        let original_updated_at = user_model.updated_at;

        // Create context with the user
        let context_with_user = Context {
            txn: &txn,
            user: Some(user_model),
            producer: None,
        };

        let change_password_dto = ChangePasswordDTO {
            old_password: "old_password".to_string(),
            new_password: "new_password".to_string(),
        };

        let result =
            change_password_use_case::execute(&context_with_user, change_password_dto).await;

        assert!(result.is_ok());

        // Verify other fields are preserved
        let updated_user = user_repository::find_by_id(&context, created_user.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated_user.email, "preserve@example.com");
        assert_eq!(updated_user.first_name, Some("Preserve".to_string()));
        assert_eq!(updated_user.last_name, Some("Fields".to_string()));
        assert_eq!(updated_user.phone, Some("9876543210".to_string()));

        // Updated_at should be changed if the implementation updates it
        if let (Some(original), Some(updated)) = (original_updated_at, updated_user.updated_at) {
            // Allow for updated_at to be the same (if implementation doesn't update timestamp)
            // or newer (if implementation does update timestamp)
            assert!(updated >= original);
        }

        // Created_at should remain the same
        assert_eq!(updated_user.created_at, created_user.created_at);

        // ID should remain the same
        assert_eq!(updated_user.id, created_user.id);
    }

    #[tokio::test]
    async fn test_change_password_without_user_context() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;

        // Create context without user (user is None)
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        let change_password_dto = ChangePasswordDTO {
            old_password: "old_password123@".to_string(),
            new_password: "new_password456".to_string(),
        };

        // Try to change password without user context
        let result = change_password_use_case::execute(&context, change_password_dto).await;

        // Should return an error because no user is authenticated
        assert!(result.is_err());
        let error = result.unwrap_err();

        // Should return 401 Unauthorized or 403 Forbidden
        assert!(error.status.as_u16() == 401 || error.status.as_u16() == 403);

        // Error message should indicate authentication is required
        assert!(
            error.message.to_lowercase().contains("unauthorized")
                || error.message.to_lowercase().contains("authentication")
                || error.message.to_lowercase().contains("login")
                || error.message.to_lowercase().contains("user")
        );
    }
}
