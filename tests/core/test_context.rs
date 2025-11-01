use std::sync::Arc;

use my_axum::core::context::Context;

use crate::setup::app::TestApp;

#[tokio::test]
async fn test_context_creation_without_user_or_producer() {
    let test_app = TestApp::spawn_db_only().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    assert!(context.user.is_none());
    assert!(context.producer.is_none());
    assert_eq!(context.locale, "en");
}

#[tokio::test]
async fn test_context_builder_accepts_optional_values() {
    let test_app = TestApp::spawn_db_only().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).locale("vi").build();

    assert_eq!(context.locale, "vi");
}

#[tokio::test]
async fn test_context_clone_keeps_transaction_handle() {
    let test_app = TestApp::spawn_db_only().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();
    let cloned = context.clone();

    assert!(cloned.user.is_none());
    assert!(std::ptr::eq(context.txn(), cloned.txn()));
}

#[tokio::test]
async fn test_context_commit_succeeds_when_uniquely_owned() {
    let test_app = TestApp::spawn_db_only().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    context.commit().await.unwrap();
}
