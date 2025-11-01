use std::sync::Arc;

use my_axum::{
    core::{context::Context, runbook},
    pkg::password,
    user::entity::{sea_orm_active_enums::UserRole, user},
};
use sea_orm::{EntityTrait, Set};

use crate::setup::app::TestApp;

#[tokio::test]
async fn test_seed_runbook_creates_default_users() {
    let test_app = TestApp::spawn_db_only().await;
    runbook::run(&test_app.setting, "seed", &[]).await.unwrap();

    let users = user::Entity::find().all(&test_app.db).await.unwrap();
    assert_eq!(users.len(), 2);
    assert!(
        users
            .iter()
            .any(|u| u.email == "admin@example.com" && u.role == UserRole::Admin)
    );
    assert!(
        users
            .iter()
            .any(|u| u.email == "user@example.com" && u.role == UserRole::User)
    );
}

#[tokio::test]
async fn test_seed_runbook_is_idempotent_and_hashes_passwords() {
    let test_app = TestApp::spawn_db_only().await;
    runbook::run(&test_app.setting, "seed", &[]).await.unwrap();
    runbook::run(&test_app.setting, "seed", &[]).await.unwrap();

    let users = user::Entity::find().all(&test_app.db).await.unwrap();
    assert_eq!(users.len(), 2);

    let regular = users
        .iter()
        .find(|u| u.email == "user@example.com")
        .unwrap();
    assert!(
        password::verify_password("password123@", &regular.password)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn test_seed_runbook_preserves_existing_user() {
    let test_app = TestApp::spawn_db_only().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    user::Entity::insert(user::ActiveModel {
        email: Set("user@example.com".to_string()),
        password: Set("existing_hash".to_string()),
        first_name: Set(Some("Existing".to_string())),
        ..Default::default()
    })
    .exec(context.txn())
    .await
    .unwrap();
    context.commit().await.unwrap();

    runbook::run(&test_app.setting, "seed", &[]).await.unwrap();

    let users = user::Entity::find().all(&test_app.db).await.unwrap();
    assert_eq!(users.len(), 2);
    let existing = users
        .iter()
        .find(|u| u.email == "user@example.com")
        .unwrap();
    assert_eq!(existing.first_name.as_deref(), Some("Existing"));
    assert_eq!(existing.password, "existing_hash");
}
