use crate::setup::app::*;

use my_axum::core::context::Context;
use my_axum::user::dto::user_dto::UserCreateDTO;
use my_axum::user::use_case::user::create_user_use_case;
use my_axum::user::use_case::user::get_user_use_case;
use sea_orm::DbErr;

#[tokio::test]
async fn test_get_user_success() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // First create a user to get
    let create_dto = UserCreateDTO {
        email: "get_test@example.com".to_string(),
        password: "TestP@ssw0rd".to_string(),
        first_name: Some("Get".to_string()),
        last_name: Some("Test".to_string()),
        phone: Some("123-456-7890".to_string()),
    };

    let created_user = create_user_use_case::execute(&context, create_dto)
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?;

    // Get the user
    let retrieved_user = get_user_use_case::execute(&context, created_user.data.id)
        .await
        .map_err(|e| DbErr::Custom(e.to_string()))?;

    // Verify the retrieved user matches what we created
    assert_eq!(retrieved_user.data.email, "get_test@example.com");
    assert_eq!(retrieved_user.data.first_name.unwrap(), "Get");
    assert_eq!(retrieved_user.data.last_name.unwrap(), "Test");
    assert_eq!(retrieved_user.data.phone.unwrap(), "123-456-7890");

    Ok(())
}

#[tokio::test]
async fn test_get_nonexistent_user() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Try to get a user with a non-existent ID
    let result = get_user_use_case::execute(&context, 999999).await;

    // Should return an error
    assert!(result.is_err());

    Ok(())
}
