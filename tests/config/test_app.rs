use my_axum::config::{app::App, setting::Setting};

use crate::setup::app::TestApp;

#[tokio::test]
async fn test_app_new_with_invalid_database_url() {
    let mut setting = Setting::new();
    setting.database_url = "invalid://invalid".to_string();

    assert!(App::new(setting).await.is_err());
}

#[tokio::test]
async fn test_app_state_is_cloneable() {
    let test_app = TestApp::spawn_db_only().await;
    let app_state = test_app.create_app_state();
    let cloned = app_state.clone();

    let _db_1 = &app_state.db;
    let _db_2 = &cloned.db;
}
