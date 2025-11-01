use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use my_axum::{
    config::app::AppState,
    core::{api::route::get_route, context::Context},
    user::entity::{refresh_token, user},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::setup::{
    app::TestApp,
    fixture::{login_admin_user, login_normal_user},
};

#[tokio::test]
async fn test_list_runbooks_requires_authentication() {
    let test_app = TestApp::spawn_db_only().await;
    let app = build_app(test_app.create_app_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/runbook/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_runbooks_returns_forbidden_for_non_admin_user() {
    let test_app = TestApp::spawn_db_only().await;
    let access_token = create_normal_access_token(&test_app).await;
    let app = build_app(test_app.create_app_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/runbook/")
                .header("authorization", format!("Bearer {access_token}"))
                .header("accept-language", "vi")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = to_json(response).await;
    let message = body["message"].as_str().unwrap();
    assert!(message.contains("Admin role is required"));
}

#[tokio::test]
async fn test_list_runbooks_returns_catalog_for_admin_user() {
    let test_app = TestApp::spawn_db_only().await;
    let access_token = create_admin_access_token(&test_app).await;
    let app = build_app(test_app.create_app_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/runbook/")
                .header("authorization", format!("Bearer {access_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_json(response).await;
    let runbooks = body["runbooks"].as_array().unwrap();
    assert!(runbooks.iter().any(|runbook| {
        runbook["name"] == "delete-refresh-tokens-by-email"
            && runbook["usage"]
                .as_str()
                .unwrap()
                .contains("runbook run delete-refresh-tokens-by-email")
    }));
}

#[tokio::test]
async fn test_run_runbook_endpoint_executes_delete_refresh_tokens_by_email_for_admin() {
    let test_app = TestApp::spawn_db_only().await;
    let access_token = create_admin_access_token(&test_app).await;

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

    let app = build_app(test_app.create_app_state());
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/runbook/run/")
        .header("authorization", format!("Bearer {access_token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "name": "delete-refresh-tokens-by-email",
                "args": ["--email", "target@example.com"],
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_json(response).await;
    assert_eq!(body["name"], "delete-refresh-tokens-by-email");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("Deleted 2 refresh token(s)")
    );

    let target_tokens = refresh_token::Entity::find()
        .filter(refresh_token::Column::UserId.eq(target_user.id))
        .all(&test_app.db)
        .await
        .unwrap();
    assert!(target_tokens.is_empty());

    let other_tokens = refresh_token::Entity::find()
        .filter(refresh_token::Column::UserId.eq(other_user.id))
        .all(&test_app.db)
        .await
        .unwrap();
    assert_eq!(other_tokens.len(), 1);
}

async fn create_admin_access_token(test_app: &TestApp) -> String {
    let txn = test_app.begin_transaction().await;
    let mut context = Context::builder(Arc::new(txn)).build();
    let (access_token, _) = login_admin_user(&mut context).await;
    context.commit().await.unwrap();
    access_token
}

async fn create_normal_access_token(test_app: &TestApp) -> String {
    let txn = test_app.begin_transaction().await;
    let mut context = Context::builder(Arc::new(txn)).build();
    let (access_token, _) = login_normal_user(&mut context).await;
    context.commit().await.unwrap();
    access_token
}

fn build_app(app_state: AppState) -> Router {
    Router::new()
        .merge(get_route(app_state.clone()))
        .with_state(app_state)
}

async fn to_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
