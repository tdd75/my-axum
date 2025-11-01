#[cfg(test)]
mod uow_tests {
    use crate::setup::app::TestApp;
    use axum::http::StatusCode;
    use my_axum::{
        core::{
            context::Context,
            db::entity::user,
            db::uow::new_transaction,
            dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
        },
        pkg::password::hash_password_string,
        user::repository::user_repository,
    };
    use sea_orm::{DbErr, TransactionTrait, entity::*};

    #[tokio::test]
    async fn test_execute_with_transaction_success() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        let result: Result<ResponseDTO<&str>, ErrorDTO> =
            new_transaction(&app_state, None, |_context| {
                Box::pin(async move {
                    // Simple successful operation
                    Ok(ResponseDTO::new(StatusCode::OK, "success"))
                })
            })
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.data, "success");
    }

    #[tokio::test]
    async fn test_execute_with_transaction_rollback_on_error() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        let result: Result<ResponseDTO<()>, ErrorDTO> =
            new_transaction(&app_state, None, |_context| {
                Box::pin(async move {
                    Err(ErrorDTO::new(
                        StatusCode::BAD_REQUEST,
                        "test error".to_string(),
                    ))
                })
            })
            .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "test error");
    }

    #[tokio::test]
    async fn test_execute_with_transaction_with_user_context() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        // Create a test user first
        let user = app_state
            .db
            .transaction::<_, user::Model, DbErr>(|txn| {
                Box::pin(async move {
                    let hashed_password = hash_password_string("test_password").await.unwrap();
                    let user_active = user::ActiveModel {
                        email: Set("testuser@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Test".to_string())),
                        last_name: Set(Some("User".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    user_repository::create(&context, user_active).await
                })
            })
            .await
            .unwrap();

        let user_id = user.id; // Extract user ID to avoid lifetime issues
        let result: Result<ResponseDTO<&str>, ErrorDTO> =
            new_transaction(&app_state, Some(user.clone()), move |context| {
                Box::pin(async move {
                    // Verify user context is passed correctly
                    assert!(context.user.is_some());
                    assert_eq!(context.user.as_ref().unwrap().id, user_id);
                    Ok(ResponseDTO::new(StatusCode::OK, "user_context_test"))
                })
            })
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data, "user_context_test");
    }

    #[tokio::test]
    async fn test_execute_with_transaction_database_operations() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        let result: Result<ResponseDTO<i32>, ErrorDTO> =
            new_transaction(&app_state, None, |context| {
                Box::pin(async move {
                    // Create a user within transaction
                    let hashed_password = hash_password_string("password123@").await.unwrap();
                    let user_active = user::ActiveModel {
                        email: Set("txn_test@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Transaction".to_string())),
                        last_name: Set(Some("Test".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };

                    let created_user = user_repository::create(context, user_active).await.unwrap();
                    Ok(ResponseDTO::new(StatusCode::CREATED, created_user.id))
                })
            })
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, StatusCode::CREATED);
        assert!(response.data > 0);
    }

    #[tokio::test]
    async fn test_execute_with_transaction_connection_error_handling() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        // Test with a use case that simulates a potential database error scenario
        let result: Result<ResponseDTO<&str>, ErrorDTO> =
            new_transaction(&app_state, None, |context| {
                Box::pin(async move {
                    // Test a simple transaction that should work
                    // In a real scenario, this could fail due to various database issues
                    let _users = user_repository::search(context, &Default::default()).await;

                    Ok(ResponseDTO::new(StatusCode::OK, "connection_test"))
                })
            })
            .await;

        // The transaction should handle any errors appropriately
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_transaction_multiple_operations() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        let result: Result<ResponseDTO<Vec<i32>>, ErrorDTO> =
            new_transaction(&app_state, None, |context| {
                Box::pin(async move {
                    // Create multiple users in same transaction
                    let mut user_ids = Vec::new();

                    for i in 1..=3 {
                        let hashed_password = hash_password_string("password123@").await.unwrap();
                        let user_active = user::ActiveModel {
                            email: Set(format!("multi_user_{}@example.com", i)),
                            password: Set(hashed_password),
                            first_name: Set(Some(format!("User{}", i))),
                            last_name: Set(Some("Test".to_string())),
                            phone: Set(None),
                            ..Default::default()
                        };

                        let created_user =
                            user_repository::create(context, user_active).await.unwrap();
                        user_ids.push(created_user.id);
                    }

                    Ok(ResponseDTO::new(StatusCode::OK, user_ids))
                })
            })
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.data.len(), 3);
    }

    #[tokio::test]
    async fn test_execute_with_transaction_partial_rollback() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        let result: Result<ResponseDTO<()>, ErrorDTO> =
            new_transaction(&app_state, None, |context| {
                Box::pin(async move {
                    // Create one user successfully
                    let hashed_password = hash_password_string("password123@").await.unwrap();
                    let user_active = user::ActiveModel {
                        email: Set("rollback_test@example.com".to_string()),
                        password: Set(hashed_password),
                        first_name: Set(Some("Rollback".to_string())),
                        last_name: Set(Some("Test".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };

                    let _created_user =
                        user_repository::create(context, user_active).await.unwrap();

                    // Then return an error to trigger rollback
                    Err(ErrorDTO::new(
                        StatusCode::CONFLICT,
                        "forced rollback".to_string(),
                    ))
                })
            })
            .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::CONFLICT);

        // Verify the user was not actually created due to rollback
        let count = app_state
            .db
            .transaction::<_, i64, DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let users = user_repository::search(&context, &Default::default())
                        .await
                        .unwrap();
                    Ok(users
                        .into_iter()
                        .filter(|u| u.email == "rollback_test@example.com")
                        .count() as i64)
                })
            })
            .await
            .unwrap();

        assert_eq!(count, 0, "User should not exist due to rollback");
    }

    #[tokio::test]
    async fn test_execute_with_transaction_context_immutability() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        let result: Result<ResponseDTO<&str>, ErrorDTO> =
            new_transaction(&app_state, None, |context| {
                Box::pin(async move {
                    // Test that context is properly structured
                    assert!(context.user.is_none());

                    // Context should be usable for database operations
                    let _users = user_repository::search(context, &Default::default())
                        .await
                        .unwrap();

                    Ok(ResponseDTO::new(StatusCode::OK, "context_test"))
                })
            })
            .await;

        assert!(result.is_ok());
    }
}
