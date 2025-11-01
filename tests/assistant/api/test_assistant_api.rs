use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use my_axum::{
    config::app::AppState,
    core::{api::route::get_route, context::Context},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::setup::{app::TestApp, fixture::login_normal_user};

#[tokio::test]
async fn test_assistant_chat_requires_authentication() {
    let test_app = TestApp::spawn_db_only().await;
    let app = build_app(test_app.create_app_state());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/assistant/chat/")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "message": "Show my profile", "messages": [] }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_assistant_chat_reports_missing_openai_key_after_auth() {
    let test_app = TestApp::spawn_db_only().await;
    let access_token = create_normal_access_token(&test_app).await;
    let mut app_state = test_app.create_app_state();
    app_state.setting.openai_api_key = None;
    let app = build_app(app_state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/assistant/chat/")
                .header("authorization", format!("Bearer {access_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "message": "Show my profile",
                        "messages": [
                            { "role": "user", "content": "What is my name?" },
                            { "role": "assistant", "content": "Your name is Admin User." }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = to_json(response).await;
    assert_eq!(body["message"], "OPENAI_API_KEY is not configured");
}

#[tokio::test]
async fn test_assistant_chat_rejects_empty_message() {
    let test_app = TestApp::spawn_db_only().await;
    let access_token = create_normal_access_token(&test_app).await;
    let app = build_app(test_app.create_app_state());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/assistant/chat/")
                .header("authorization", format!("Bearer {access_token}"))
                .header("content-type", "application/json")
                .body(Body::from(json!({ "message": "   " }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_json(response).await;
    assert_eq!(body["message"], "Message is required");
}

#[tokio::test]
async fn test_assistant_chat_rejects_too_much_context() {
    let test_app = TestApp::spawn_db_only().await;
    let access_token = create_normal_access_token(&test_app).await;
    let app = build_app(test_app.create_app_state());
    let messages = (0..13)
        .map(|index| json!({ "role": "user", "content": format!("message {index}") }))
        .collect::<Vec<_>>();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/assistant/chat/")
                .header("authorization", format!("Bearer {access_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "message": "continue",
                        "messages": messages,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_json(response).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("at most 12 messages")
    );
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
