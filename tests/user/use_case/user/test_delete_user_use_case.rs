use my_axum::user::dto::user_dto::UserCreateDTO;
use my_axum::user::use_case::user::{delete_user_use_case, get_user_use_case};
use sea_orm::DbErr;

use crate::setup::app::TestApp;

mod delete_user_tests {
    use my_axum::{core::context::Context, user::use_case::user::create_user_use_case};

    use super::*;

    #[tokio::test]
    async fn should_delete_user_successfully() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // First, create a user
        let dto = UserCreateDTO {
            email: "delete_test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Bob".to_string()),
            last_name: Some("Johnson".to_string()),
            phone: Some("555-123-4567".to_string()),
        };

        let created_user = create_user_use_case::execute(&context, dto)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        // Delete the user
        let delete_result = delete_user_use_case::execute(&context, created_user.data.id).await;

        match delete_result {
            Ok(_) => {
                // Verify the user no longer exists
                let read_result = get_user_use_case::execute(&context, created_user.data.id).await;
                match read_result {
                    Ok(_) => Err(DbErr::Custom(
                        "User should not exist after deletion".to_string(),
                    )),
                    Err(_) => Ok(()), // Expected: user not found
                }
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }
}
