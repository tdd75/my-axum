use chrono::{Duration, Utc};
use my_axum::{
    core::runbook,
    user::entity::{refresh_token, user},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::setup::app::TestApp;

#[tokio::test]
async fn test_runbook_rejects_unknown_names() {
    let test_app = TestApp::spawn_db_only().await;

    let error = runbook::run(&test_app.setting, "missing-runbook", &[])
        .await
        .unwrap_err();
    assert!(error.to_string().contains("Unknown runbook"));
}

#[tokio::test]
async fn test_seed_runbook_executes_without_arguments() {
    let test_app = TestApp::spawn_db_only().await;

    let result = runbook::run(&test_app.setting, "seed", &[]).await.unwrap();

    assert_eq!(result.name, "seed");
    assert!(result.message.contains("Seeded"));
}

#[tokio::test]
async fn test_delete_refresh_tokens_runbook_validates_arguments() {
    let test_app = TestApp::spawn_db_only().await;

    let error = runbook::run(
        &test_app.setting,
        "delete-refresh-tokens-by-email",
        &["--email".to_string()],
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("Missing value"));
}

#[tokio::test]
async fn test_delete_refresh_tokens_runbook_removes_only_target_users_tokens() {
    let test_app = TestApp::spawn_db_only().await;

    let target_user = user::Entity::insert(user::ActiveModel {
        email: Set("target@example.com".to_string()),
        password: Set("hashed".to_string()),
        ..Default::default()
    })
    .exec_with_returning(&test_app.db)
    .await
    .unwrap();
    let other_user = user::Entity::insert(user::ActiveModel {
        email: Set("other@example.com".to_string()),
        password: Set("hashed".to_string()),
        ..Default::default()
    })
    .exec_with_returning(&test_app.db)
    .await
    .unwrap();

    for token in ["target_token_1", "target_token_2"] {
        refresh_token::Entity::insert(refresh_token::ActiveModel {
            user_id: Set(target_user.id),
            token: Set(token.to_string()),
            expires_at: Set((Utc::now() + Duration::hours(1)).naive_utc()),
            ..Default::default()
        })
        .exec(&test_app.db)
        .await
        .unwrap();
    }
    refresh_token::Entity::insert(refresh_token::ActiveModel {
        user_id: Set(other_user.id),
        token: Set("other_token".to_string()),
        expires_at: Set((Utc::now() + Duration::hours(1)).naive_utc()),
        ..Default::default()
    })
    .exec(&test_app.db)
    .await
    .unwrap();

    runbook::run(
        &test_app.setting,
        "delete-refresh-tokens-by-email",
        &["--email".to_string(), "target@example.com".to_string()],
    )
    .await
    .unwrap();

    let target_tokens = refresh_token::Entity::find()
        .filter(refresh_token::Column::UserId.eq(target_user.id))
        .all(&test_app.db)
        .await
        .unwrap();
    let other_tokens = refresh_token::Entity::find()
        .filter(refresh_token::Column::UserId.eq(other_user.id))
        .all(&test_app.db)
        .await
        .unwrap();

    assert!(target_tokens.is_empty());
    assert_eq!(other_tokens.len(), 1);
}
