use crate::setup::app::TestApp;

use my_axum::core::db::entity::user;
use my_axum::user::dto::user_dto::UserCreateDTO;
use my_axum::user::service::user_service;
use my_axum::{core::context::Context, user::use_case::user::create_user_use_case};
use sea_orm::DbErr;

#[tokio::test]
async fn test_read_user_service_success() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;

    // Create a user first
    let dto = UserCreateDTO {
        email: "service_test@example.com".to_string(),
        password: "password123@".to_string(),
        first_name: Some("Service".to_string()),
        last_name: Some("Test".to_string()),
        phone: Some("123-456-7890".to_string()),
    };

    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };
    let created_user = create_user_use_case::execute(&context, dto).await.unwrap();

    // Test read service
    let result = user_service::read(&context, created_user.data.id).await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, "service_test@example.com");

    Ok(())
}

#[tokio::test]
async fn test_read_user_service_not_found() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;

    // Test with non-existent ID
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };
    let result = user_service::read(&context, 999999).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));

    Ok(())
}

#[tokio::test]
async fn test_validate_unique_email_success() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;

    // Test with a unique email
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };
    let result = user_service::validate_unique_email(&context, "unique@example.com", None).await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_validate_unique_email_duplicate() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;

    // Create a user first
    let dto = UserCreateDTO {
        email: "existing@example.com".to_string(),
        password: "password123@".to_string(),
        first_name: Some("Existing".to_string()),
        last_name: Some("User".to_string()),
        phone: Some("123-456-7890".to_string()),
    };

    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };
    create_user_use_case::execute(&context, dto).await.unwrap();

    // Test with existing email
    let result = user_service::validate_unique_email(&context, "existing@example.com", None).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    Ok(())
}

#[tokio::test]
async fn test_validate_unique_email_exclude_self() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;

    // Create a user first
    let dto = UserCreateDTO {
        email: "exclude_test@example.com".to_string(),
        password: "password123@".to_string(),
        first_name: Some("Exclude".to_string()),
        last_name: Some("Test".to_string()),
        phone: Some("123-456-7890".to_string()),
    };

    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };
    let created_user = create_user_use_case::execute(&context, dto).await.unwrap();

    // Test with same email but excluding the user's own ID (should pass)
    let result = user_service::validate_unique_email(
        &context,
        "exclude_test@example.com",
        Some(created_user.data.id),
    )
    .await;

    assert!(result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_validate_unique_email_exclude_different_id() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;

    // Create a user first
    let dto = UserCreateDTO {
        email: "exclude_diff@example.com".to_string(),
        password: "password123@".to_string(),
        first_name: Some("Different".to_string()),
        last_name: Some("User".to_string()),
        phone: Some("123-456-7890".to_string()),
    };

    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };
    create_user_use_case::execute(&context, dto).await.unwrap();

    // Test with same email but excluding a different ID (should fail)
    let result =
        user_service::validate_unique_email(&context, "exclude_diff@example.com", Some(999999))
            .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    Ok(())
}

#[tokio::test]
async fn test_fetch_created_users_success() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;

    // Create users first
    let creator_dto = UserCreateDTO {
        email: "creator@example.com".to_string(),
        password: "password123@".to_string(),
        first_name: Some("Creator".to_string()),
        last_name: Some("User".to_string()),
        phone: Some("123-456-7890".to_string()),
    };

    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };
    let creator = create_user_use_case::execute(&context, creator_dto)
        .await
        .unwrap();

    // Create another user that references the creator
    let user_dto = UserCreateDTO {
        email: "created_by@example.com".to_string(),
        password: "password123@".to_string(),
        first_name: Some("Created".to_string()),
        last_name: Some("By".to_string()),
        phone: Some("123-456-7890".to_string()),
    };
    let user = create_user_use_case::execute(&context, user_dto)
        .await
        .unwrap();

    // Manually set created_user_id
    use my_axum::core::db::entity::user::{ActiveModel, Entity as UserEntity};
    use sea_orm::{ActiveModelTrait, EntityTrait, Set};

    // Fetch the user entity from database
    let user_entity = UserEntity::find_by_id(user.data.id)
        .one(context.txn)
        .await
        .unwrap()
        .unwrap();

    let mut active_user: ActiveModel = user_entity.into();
    active_user.created_user_id = Set(Some(creator.data.id));
    let updated_user = active_user.update(context.txn).await.unwrap();

    // Test build_user_map with created users
    let users: Vec<user::Model> = vec![updated_user];
    let created_user_ids: Vec<i32> = users.iter().filter_map(|u| u.created_user_id).collect();
    let result = user_service::build_user_map(&context, &created_user_ids).await;

    assert!(result.is_ok());
    let created_users_map = result.unwrap();
    assert_eq!(created_users_map.len(), 1);
    assert!(created_users_map.contains_key(&creator.data.id));

    Ok(())
}

#[tokio::test]
async fn test_fetch_created_users_empty() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;

    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Test with empty users list
    let users: Vec<user::Model> = vec![];
    let created_user_ids: Vec<i32> = users.iter().filter_map(|u| u.created_user_id).collect();
    let result = user_service::build_user_map(&context, &created_user_ids).await;

    assert!(result.is_ok());
    let created_users_map = result.unwrap();
    assert_eq!(created_users_map.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_fetch_updated_users_success() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;

    // Create users first
    let updater_dto = UserCreateDTO {
        email: "updater@example.com".to_string(),
        password: "password123@".to_string(),
        first_name: Some("Updater".to_string()),
        last_name: Some("User".to_string()),
        phone: Some("123-456-7890".to_string()),
    };

    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };
    let updater = create_user_use_case::execute(&context, updater_dto)
        .await
        .unwrap();

    // Create another user that references the updater
    let user_dto = UserCreateDTO {
        email: "updated_by@example.com".to_string(),
        password: "password123@".to_string(),
        first_name: Some("Updated".to_string()),
        last_name: Some("By".to_string()),
        phone: Some("123-456-7890".to_string()),
    };
    let user = create_user_use_case::execute(&context, user_dto)
        .await
        .unwrap();

    // Manually set updated_user_id
    use my_axum::core::db::entity::user::{ActiveModel as UserActiveModel, Entity as UserEntity2};
    use sea_orm::{ActiveModelTrait as _, EntityTrait as _, Set as UserSet};

    // Fetch the user entity from database
    let user_entity = UserEntity2::find_by_id(user.data.id)
        .one(context.txn)
        .await
        .unwrap()
        .unwrap();

    let mut active_user: UserActiveModel = user_entity.into();
    active_user.updated_user_id = UserSet(Some(updater.data.id));
    let updated_user = active_user.update(context.txn).await.unwrap();

    // Test build_user_map with updated users
    let users: Vec<my_axum::core::db::entity::user::Model> = vec![updated_user];
    let updated_user_ids: Vec<i32> = users.iter().filter_map(|u| u.updated_user_id).collect();
    let result = user_service::build_user_map(&context, &updated_user_ids).await;

    assert!(result.is_ok());
    let updated_users_map = result.unwrap();
    assert_eq!(updated_users_map.len(), 1);
    assert!(updated_users_map.contains_key(&updater.data.id));

    Ok(())
}

#[tokio::test]
async fn test_fetch_updated_users_empty() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;

    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Test with empty users list
    let users: Vec<my_axum::core::db::entity::user::Model> = vec![];
    let updated_user_ids: Vec<i32> = users.iter().filter_map(|u| u.updated_user_id).collect();
    let result = user_service::build_user_map(&context, &updated_user_ids).await;

    assert!(result.is_ok());
    let updated_users_map = result.unwrap();
    assert_eq!(updated_users_map.len(), 0);

    Ok(())
}
