use std::sync::Arc;

use axum::http::StatusCode;
use my_axum::{
    config::{app::AppState, setting::Setting},
    core::{
        context::Context,
        db::{connection::get_db, entity::user, uow::new_transaction},
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    pkg::password::hash_password_string,
    user::repository::user_repository,
};
use sea_orm::{ActiveValue::Set, DbErr, TransactionTrait};

use crate::setup::app::TestApp;

#[tokio::test]
async fn test_new_transaction_commits_successful_result() {
    let test_app = TestApp::spawn_db_only().await;
    let app_state = test_app.create_app_state();

    let result: Result<ResponseDTO<&str>, ErrorDTO> =
        new_transaction(&app_state, None, None, |_| {
            Box::pin(async move { Ok(ResponseDTO::new(StatusCode::OK, "success")) })
        })
        .await;

    assert_eq!(result.unwrap().data, "success");
}

#[tokio::test]
async fn test_new_transaction_rolls_back_on_error() {
    let test_app = TestApp::spawn_db_only().await;
    let app_state = test_app.create_app_state();

    let result: Result<ResponseDTO<()>, ErrorDTO> = new_transaction(&app_state, None, None, |_| {
        Box::pin(async move {
            Err(ErrorDTO::new(
                StatusCode::BAD_REQUEST,
                "forced failure".to_string(),
            ))
        })
    })
    .await;

    let error = result.unwrap_err();
    assert_eq!(error.status, StatusCode::BAD_REQUEST);
    assert_eq!(error.message, "forced failure");
}

#[tokio::test]
async fn test_new_transaction_passes_current_user_into_context() {
    let test_app = TestApp::spawn_db_only().await;
    let app_state = test_app.create_app_state();

    let created_user = app_state
        .db
        .transaction::<_, user::Model, DbErr>(|txn| {
            Box::pin(async move {
                let context = Context::builder(Arc::new(txn.begin().await?)).build();
                let hashed_password = hash_password_string("password123@").await.unwrap();
                let user = user::ActiveModel {
                    email: Set("context-user@example.com".to_string()),
                    password: Set(hashed_password),
                    ..Default::default()
                };

                user_repository::create(&context, user).await
            })
        })
        .await
        .unwrap();

    let user_id = created_user.id;
    let result: Result<ResponseDTO<i32>, ErrorDTO> =
        new_transaction(&app_state, Some(created_user), None, |context| {
            Box::pin(async move {
                assert_eq!(context.user.as_ref().unwrap().id, user_id);
                Ok(ResponseDTO::new(StatusCode::OK, user_id))
            })
        })
        .await;

    assert_eq!(result.unwrap().data, user_id);
}

#[tokio::test]
async fn test_new_transaction_persists_changes_only_on_success() {
    let test_app = TestApp::spawn_db_only().await;
    let app_state = test_app.create_app_state();

    let result: Result<ResponseDTO<()>, ErrorDTO> =
        new_transaction(&app_state, None, None, |context| {
            Box::pin(async move {
                let hashed_password = hash_password_string("password123@").await.unwrap();
                let user = user::ActiveModel {
                    email: Set("rollback-test@example.com".to_string()),
                    password: Set(hashed_password),
                    ..Default::default()
                };
                user_repository::create(context, user).await.unwrap();

                Err(ErrorDTO::new(StatusCode::CONFLICT, "rollback".to_string()))
            })
        })
        .await;

    assert_eq!(result.unwrap_err().status, StatusCode::CONFLICT);

    let existing = app_state
        .db
        .transaction::<_, Option<user::Model>, DbErr>(|txn| {
            Box::pin(async move {
                let context = Context::builder(Arc::new(txn.begin().await?)).build();
                user_repository::find_by_email(&context, "rollback-test@example.com").await
            })
        })
        .await
        .unwrap();

    assert!(existing.is_none());
}

#[tokio::test]
async fn test_new_transaction_surfaces_begin_transaction_errors() {
    let db = get_db("sqlite::memory:").await.unwrap();
    let app_state = AppState {
        db: db.clone(),
        setting: Setting::new(),
        producer: None,
    };
    db.close().await.unwrap();

    let result: Result<ResponseDTO<&str>, ErrorDTO> =
        new_transaction(&app_state, None, None, |_| {
            Box::pin(async move { Ok(ResponseDTO::new(StatusCode::OK, "unreachable")) })
        })
        .await;

    assert_eq!(
        result.unwrap_err().status,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}
