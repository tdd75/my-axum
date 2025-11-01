use my_axum::user::dto::user_dto::{UserCreateDTO, UserUpdateDTO};
use my_axum::user::use_case::user::update_user_use_case;
use sea_orm::DbErr;

use crate::setup::app::TestApp;

mod update_user_tests {
    use my_axum::{core::context::Context, user::use_case::user::create_user_use_case};
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn should_update_all_user_fields() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        // Create initial user
        let create_dto = UserCreateDTO {
            email: "update_all@example.com".to_string(),
            password: "old_password".to_string(),
            first_name: Some("Old".to_string()),
            last_name: Some("Name".to_string()),
            phone: Some("123-456-7890".to_string()),
        };

        let created_user = create_user_use_case::execute(&context, create_dto)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        let initial_updated_at = created_user.data.updated_at;

        // Update all fields
        let update_dto = UserUpdateDTO {
            email: Some("new_email@example.com".to_string()),
            password: Some("NewP@ssw0rd".to_string()),
            first_name: Some("New".to_string()),
            last_name: Some("Updated".to_string()),
            phone: Some("098-765-4321".to_string()),
        };

        let updated_user = update_user_use_case::execute(
            &context,
            created_user.data.id,
            update_dto,
            vec![
                "email".to_string(),
                "password".to_string(),
                "first_name".to_string(),
                "last_name".to_string(),
                "phone".to_string(),
            ],
        )
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?;

        // Verify all fields were updated
        assert_eq!(updated_user.data.email, "new_email@example.com");
        assert_eq!(updated_user.data.first_name.unwrap(), "New");
        assert_eq!(updated_user.data.last_name.unwrap(), "Updated");
        assert_eq!(updated_user.data.phone.unwrap(), "098-765-4321");

        // Verify updated_at was changed
        assert_ne!(updated_user.data.updated_at, initial_updated_at);
        assert!(updated_user.data.updated_at > initial_updated_at);

        Ok(())
    }

    #[tokio::test]
    async fn should_return_not_found_for_nonexistent_user() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        let update_dto = UserUpdateDTO {
            email: Some("new@example.com".to_string()),
            password: None,
            first_name: None,
            last_name: None,
            phone: None,
        };

        let result =
            update_user_use_case::execute(&context, 999999, update_dto, vec!["email".to_string()])
                .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 404);

        Ok(())
    }

    #[tokio::test]
    async fn should_reject_duplicate_email() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        // Create two users
        let user1 = create_user_use_case::execute(
            &context,
            UserCreateDTO {
                email: "user1_dup@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("User1".to_string()),
                last_name: None,
                phone: None,
            },
        )
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?;

        let _user2 = create_user_use_case::execute(
            &context,
            UserCreateDTO {
                email: "user2_dup@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("User2".to_string()),
                last_name: None,
                phone: None,
            },
        )
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?;

        // Try to update user1's email to user2's email
        let update_dto = UserUpdateDTO {
            email: Some("user2_dup@example.com".to_string()),
            password: None,
            first_name: None,
            last_name: None,
            phone: None,
        };

        let result = update_user_use_case::execute(
            &context,
            user1.data.id,
            update_dto,
            vec!["email".to_string()],
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 409);

        Ok(())
    }

    #[tokio::test]
    async fn should_reject_weak_password() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        let created = create_user_use_case::execute(
            &context,
            UserCreateDTO {
                email: "weak_pw@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Test".to_string()),
                last_name: None,
                phone: None,
            },
        )
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?;

        let update_dto = UserUpdateDTO {
            email: None,
            password: Some("123".to_string()), // Too weak
            first_name: None,
            last_name: None,
            phone: None,
        };

        let result = update_user_use_case::execute(
            &context,
            created.data.id,
            update_dto,
            vec!["password".to_string()],
        )
        .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);

        Ok(())
    }

    #[tokio::test]
    async fn should_update_single_field_only() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        let created = create_user_use_case::execute(
            &context,
            UserCreateDTO {
                email: "single_field@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Original".to_string()),
                last_name: Some("Last".to_string()),
                phone: Some("1234567890".to_string()),
            },
        )
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?;

        // Update only last_name, phone via fields vec
        let update_dto = UserUpdateDTO {
            email: None,
            password: None,
            first_name: None,
            last_name: Some("NewLast".to_string()),
            phone: Some("9999999999".to_string()),
        };

        let result = update_user_use_case::execute(
            &context,
            created.data.id,
            update_dto,
            vec!["last_name".to_string(), "phone".to_string()],
        )
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(result.data.last_name, Some("NewLast".to_string()));
        assert_eq!(result.data.phone, Some("9999999999".to_string()));
        // email and first_name should remain unchanged
        assert_eq!(result.data.email, "single_field@example.com");
        assert_eq!(result.data.first_name, Some("Original".to_string()));

        Ok(())
    }
}
