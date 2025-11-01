mod app_initialization_tests {
    use my_axum::config::{app::App, setting::Setting};

    use crate::setup::app::TestApp;

    #[tokio::test]
    async fn test_app_new_success() {
        let test_app = TestApp::spawn_app().await;

        // Verify the app properties through TestApp
        assert!(!test_app.base_url.is_empty());
        assert!(
            test_app.base_url.contains(":"),
            "Base URL should contain a port"
        );
        assert!(!test_app.db_url.is_empty());
    }

    #[tokio::test]
    async fn test_app_new_with_invalid_database_url() {
        let mut setting = Setting::new();
        setting.database_url = "invalid://invalid".to_string();

        let app = App::new(setting).await;
        assert!(app.is_err());
    }
}

mod transaction_tests {
    use http::StatusCode;
    use my_axum::{
        config::{app::AppState, setting::Setting},
        core::{
            context::Context,
            db::connection::get_db,
            db::uow::new_transaction,
            dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
        },
    };
    use sea_orm::{DbErr, TransactionTrait};

    use crate::setup::app::TestApp;

    #[tokio::test]
    async fn test_execute_with_transaction_success() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        let result: Result<ResponseDTO<String>, ErrorDTO> =
            new_transaction(&app_state, None, |_context| {
                Box::pin(
                    async move { Ok(ResponseDTO::new(StatusCode::OK, "test_data".to_string())) },
                )
            })
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.data, "test_data");
    }

    #[tokio::test]
    async fn test_execute_with_transaction_error_rollback() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();

        let result: Result<ResponseDTO<String>, ErrorDTO> =
            new_transaction(&app_state, None, |_context| {
                Box::pin(async move {
                    Err(ErrorDTO::new(
                        StatusCode::BAD_REQUEST,
                        "Test error".to_string(),
                    ))
                })
            })
            .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Test error");
    }

    #[tokio::test]
    async fn test_execute_with_transaction_with_user_context() {
        use my_axum::{
            core::db::entity::user, pkg::password::hash_password_string,
            user::repository::user_repository,
        };
        use sea_orm::ActiveValue::Set;

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
                    let user = user_repository::create(&context, user_active)
                        .await
                        .unwrap();
                    Ok(user)
                })
            })
            .await
            .unwrap();

        let user_id = user.id;
        let result: Result<ResponseDTO<i32>, ErrorDTO> =
            new_transaction(&app_state, Some(user.clone()), move |context| {
                let expected_user = user.clone();
                Box::pin(async move {
                    assert!(context.user.is_some());
                    assert_eq!(context.user.as_ref().unwrap().id, expected_user.id);

                    Ok(ResponseDTO::new(StatusCode::OK, expected_user.id))
                })
            })
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.data, user_id);
    }

    #[tokio::test]
    async fn test_execute_with_transaction_begin_transaction_error() {
        // Create a closed database connection to simulate error
        let db = get_db("sqlite::memory:").await.unwrap();
        let setting = Setting::new();
        let app_state = AppState {
            db: db.clone(),
            setting,
            producer: None,
        };
        db.close().await.unwrap();

        let result: Result<ResponseDTO<String>, ErrorDTO> =
            new_transaction(&app_state, None, |_context| {
                Box::pin(async move {
                    Ok(ResponseDTO::new(
                        StatusCode::OK,
                        "Should not reach here".to_string(),
                    ))
                })
            })
            .await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}

mod context_tests {
    use my_axum::{
        core::{context::Context, db::entity::user},
        pkg::password::hash_password_string,
        user::repository::user_repository,
    };
    use sea_orm::{ActiveValue::Set, TransactionTrait};

    use crate::setup::app::TestApp;

    #[tokio::test]
    async fn test_context_struct_fields() {
        let test_app = TestApp::spawn_app().await;

        let txn = test_app.db.begin().await.unwrap();

        // Create a test user
        let hashed_password = hash_password_string("test_password").await.unwrap();
        let user_active = user::ActiveModel {
            email: Set("contextuser@example.com".to_string()),
            password: Set(hashed_password),
            first_name: Set(Some("Context".to_string())),
            last_name: Set(Some("User".to_string())),
            phone: Set(None),
            ..Default::default()
        };
        let temp_context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let user = user_repository::create(&temp_context, user_active)
            .await
            .unwrap();

        let context = Context {
            txn: &txn,
            user: Some(user.clone()),
            producer: None,
        };

        assert!(context.user.is_some());
        assert_eq!(context.user.as_ref().unwrap().id, user.id);

        txn.rollback().await.unwrap();
    }
}

mod app_state_tests {
    use crate::setup::app::TestApp;

    #[tokio::test]
    async fn test_app_state_clone() {
        let test_app = TestApp::spawn_app().await;
        let app_state = test_app.create_app_state();
        let cloned_state = app_state.clone();

        // Verify both states work
        let _connection1 = &app_state.db;
        let _connection2 = &cloned_state.db;

        // Test passes if clone succeeds without panic
    }
}
