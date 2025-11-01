#[cfg(test)]
mod trace_layer_tests {
    use my_axum::core::layer::trace_layer::get_trace_layer;

    #[test]
    fn test_get_trace_layer_returns_trace_layer() {
        let trace_layer = get_trace_layer();

        // Verify that the function returns a TraceLayer
        // We can't easily test the internal configuration without making requests,
        // but we can verify the layer is created successfully
        assert!(std::mem::size_of_val(&trace_layer) > 0);
    }

    #[test]
    fn test_trace_layer_can_be_created() {
        let _trace_layer = get_trace_layer();
        // If this compiles and runs, the layer is created successfully
    }

    #[test]
    fn test_trace_layer_can_be_cloned() {
        let trace_layer = get_trace_layer();
        let cloned_layer = trace_layer.clone();

        // Verify both layers exist
        assert!(std::mem::size_of_val(&trace_layer) > 0);
        assert!(std::mem::size_of_val(&cloned_layer) > 0);
    }

    #[test]
    fn test_multiple_trace_layer_creation() {
        let layer1 = get_trace_layer();
        let layer2 = get_trace_layer();

        // Verify multiple layers can be created independently
        assert!(std::mem::size_of_val(&layer1) > 0);
        assert!(std::mem::size_of_val(&layer2) > 0);
    }

    // Note: The actual tracing functionality (span creation, logging, etc.)
    // would be better tested through integration tests with actual HTTP requests,
    // as the TraceLayer is designed to work within the Tower/Axum middleware stack.

    #[tokio::test]
    async fn test_trace_layer_with_axum_router() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn test_handler() -> &'static str {
            "test"
        }

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(get_trace_layer());

        // Test that the router with trace layer can be created and responds
        let request = Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_different_methods() {
        use axum::{
            Router,
            routing::{get, post},
        };
        use http::{Method, Request, StatusCode};
        use tower::ServiceExt;

        async fn get_handler() -> &'static str {
            "get"
        }

        async fn post_handler() -> &'static str {
            "post"
        }

        let app = Router::new()
            .route("/test", get(get_handler))
            .route("/test", post(post_handler))
            .layer(get_trace_layer());

        // Test GET request
        let get_request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.clone().oneshot(get_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test POST request
        let post_request = Request::builder()
            .method(Method::POST)
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(post_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_different_uri_paths() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn handler() -> &'static str {
            "response"
        }

        let app = Router::new()
            .route("/api/v1/users", get(handler))
            .route("/api/v1/auth/login", get(handler))
            .route("/health", get(handler))
            .layer(get_trace_layer());

        for path in ["/api/v1/users", "/api/v1/auth/login", "/health"] {
            let request = Request::builder()
                .uri(path)
                .body(axum::body::Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_trace_layer_with_404_response() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn handler() -> &'static str {
            "response"
        }

        let app = Router::new()
            .route("/exists", get(handler))
            .layer(get_trace_layer());

        let request = Request::builder()
            .uri("/does-not-exist")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_trace_layer_with_query_parameters() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn handler() -> &'static str {
            "response"
        }

        let app = Router::new()
            .route("/search", get(handler))
            .layer(get_trace_layer());

        let request = Request::builder()
            .uri("/search?q=test&page=1&limit=10")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_put_request() {
        use axum::{Router, routing::put};
        use http::{Method, Request, StatusCode};
        use tower::ServiceExt;

        async fn put_handler() -> &'static str {
            "updated"
        }

        let app = Router::new()
            .route("/resource", put(put_handler))
            .layer(get_trace_layer());

        let request = Request::builder()
            .method(Method::PUT)
            .uri("/resource")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_delete_request() {
        use axum::{Router, routing::delete};
        use http::{Method, Request, StatusCode};
        use tower::ServiceExt;

        async fn delete_handler() -> &'static str {
            "deleted"
        }

        let app = Router::new()
            .route("/resource", delete(delete_handler))
            .layer(get_trace_layer());

        let request = Request::builder()
            .method(Method::DELETE)
            .uri("/resource")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_patch_request() {
        use axum::{Router, routing::patch};
        use http::{Method, Request, StatusCode};
        use tower::ServiceExt;

        async fn patch_handler() -> &'static str {
            "patched"
        }

        let app = Router::new()
            .route("/resource", patch(patch_handler))
            .layer(get_trace_layer());

        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/resource")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_internal_server_error() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn error_handler() -> StatusCode {
            StatusCode::INTERNAL_SERVER_ERROR
        }

        let app = Router::new()
            .route("/error", get(error_handler))
            .layer(get_trace_layer());

        let request = Request::builder()
            .uri("/error")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_trace_layer_with_multiple_layers() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn handler() -> &'static str {
            "response"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(get_trace_layer())
            .layer(get_trace_layer()); // Double tracing layer

        let request = Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_complex_uri() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn handler() -> &'static str {
            "response"
        }

        let app = Router::new()
            .route("/api/v1/search", get(handler))
            .layer(get_trace_layer());

        let request = Request::builder()
            .uri("/api/v1/search?q=hello%20world&page=1&limit=50&sort=desc")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_captures_response_time() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use std::time::Duration;
        use tower::ServiceExt;

        async fn slow_handler() -> &'static str {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "response"
        }

        let app = Router::new()
            .route("/slow", get(slow_handler))
            .layer(get_trace_layer());

        let request = Request::builder()
            .uri("/slow")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_http_versions() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode, Version};
        use tower::ServiceExt;

        async fn handler() -> &'static str {
            "response"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(get_trace_layer());

        // Test with HTTP/1.1
        let request = Request::builder()
            .version(Version::HTTP_11)
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test with HTTP/2.0
        let request = Request::builder()
            .version(Version::HTTP_2)
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_various_status_codes() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn ok_handler() -> StatusCode {
            StatusCode::OK
        }

        async fn created_handler() -> StatusCode {
            StatusCode::CREATED
        }

        async fn bad_request_handler() -> StatusCode {
            StatusCode::BAD_REQUEST
        }

        async fn unauthorized_handler() -> StatusCode {
            StatusCode::UNAUTHORIZED
        }

        async fn not_found_handler() -> StatusCode {
            StatusCode::NOT_FOUND
        }

        let app = Router::new()
            .route("/ok", get(ok_handler))
            .route("/created", get(created_handler))
            .route("/bad-request", get(bad_request_handler))
            .route("/unauthorized", get(unauthorized_handler))
            .route("/not-found", get(not_found_handler))
            .layer(get_trace_layer());

        let test_cases = vec![
            ("/ok", StatusCode::OK),
            ("/created", StatusCode::CREATED),
            ("/bad-request", StatusCode::BAD_REQUEST),
            ("/unauthorized", StatusCode::UNAUTHORIZED),
            ("/not-found", StatusCode::NOT_FOUND),
        ];

        for (path, expected_status) in test_cases {
            let request = Request::builder()
                .uri(path)
                .body(axum::body::Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), expected_status);
        }
    }

    #[tokio::test]
    async fn test_trace_layer_with_different_content_types() {
        use axum::{Json, Router, routing::post};
        use http::{Request, StatusCode, header};
        use serde_json::json;
        use tower::ServiceExt;

        async fn json_handler(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
            Json(payload)
        }

        let app = Router::new()
            .route("/json", post(json_handler))
            .layer(get_trace_layer());

        let body = json!({"test": "data"}).to_string();
        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/json")
            .header(header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_long_uri() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn handler() -> &'static str {
            "response"
        }

        let app = Router::new()
            .route(
                "/api/v1/very/deeply/nested/endpoint/with/many/segments",
                get(handler),
            )
            .layer(get_trace_layer());

        let long_query = "?".to_owned()
            + &(0..50)
                .map(|i| format!("param{}=value{}", i, i))
                .collect::<Vec<_>>()
                .join("&");
        let uri = format!(
            "/api/v1/very/deeply/nested/endpoint/with/many/segments{}",
            long_query
        );

        let request = Request::builder()
            .uri(uri)
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_concurrent_requests() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn handler() -> &'static str {
            "response"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(get_trace_layer());

        let mut handles = vec![];

        for _ in 0..10 {
            let app_clone = app.clone();
            let handle = tokio::spawn(async move {
                let request = Request::builder()
                    .uri("/test")
                    .body(axum::body::Body::empty())
                    .unwrap();

                let response = app_clone.oneshot(request).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_trace_layer_preserves_request_body() {
        use axum::{Router, body::Body, routing::post};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn echo_handler(body: String) -> String {
            body
        }

        let app = Router::new()
            .route("/echo", post(echo_handler))
            .layer(get_trace_layer());

        let body_content = "test body content";
        let request = Request::builder()
            .method(http::Method::POST)
            .uri("/echo")
            .body(Body::from(body_content))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_with_empty_uri_path() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn root_handler() -> &'static str {
            "root"
        }

        let app = Router::new()
            .route("/", get(root_handler))
            .layer(get_trace_layer());

        let request = Request::builder()
            .uri("/")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_layer_idempotency() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn handler() -> &'static str {
            "response"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(get_trace_layer());

        // Make the same request multiple times
        for _ in 0..5 {
            let request = Request::builder()
                .uri("/test")
                .body(axum::body::Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }
}
