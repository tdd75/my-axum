use my_axum::user::dto::user_dto::{UserCreateDTO, UserUpdateDTO};
use my_axum::user::use_case::user::update_user_use_case;
use sea_orm::DbErr;

use crate::setup::app::TestApp;

mod update_user_tests {
    use my_axum::{core::context::Context, user::use_case::user::create_user_use_case};

    use super::*;

    #[tokio::test]
    async fn should_update_all_user_fields() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

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
}
