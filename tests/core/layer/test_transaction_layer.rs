use axum::{
    Router,
    body::Body,
    extract::Extension,
    http::{Request, StatusCode},
    middleware,
    routing::get,
};
use my_axum::core::{context::Context, layer::transaction_layer::transaction_middleware};
use tower::ServiceExt;

use crate::setup::app::TestApp;

#[tokio::test]
async fn test_transaction_middleware_injects_context() {
    let test_app = TestApp::spawn_db_only().await;
    let app_state = test_app.create_app_state();

    let app = Router::new()
        .route(
            "/test",
            get(|Extension(ctx): Extension<Context>| async move {
                assert!(ctx.user.is_none());
                StatusCode::OK
            }),
        )
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            transaction_middleware,
        ))
        .with_state(app_state);

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_transaction_middleware_preserves_error_statuses() {
    let test_app = TestApp::spawn_db_only().await;
    let app_state = test_app.create_app_state();

    let app =
        Router::new()
            .route(
                "/test",
                get(|Extension(_ctx): Extension<Context>| async move {
                    StatusCode::INTERNAL_SERVER_ERROR
                }),
            )
            .route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                transaction_middleware,
            ))
            .with_state(app_state);

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_transaction_middleware_handles_cloned_context() {
    let test_app = TestApp::spawn_db_only().await;
    let app_state = test_app.create_app_state();

    let app = Router::new()
        .route(
            "/test",
            get(|Extension(ctx): Extension<Context>| async move {
                let _clone = ctx.clone();
                StatusCode::FOUND
            }),
        )
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            transaction_middleware,
        ))
        .with_state(app_state);

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FOUND);
}
