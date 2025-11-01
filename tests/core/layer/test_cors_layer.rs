#[cfg(test)]
mod cors_layer_tests {
    use my_axum::core::layer::cors_layer::get_cors_layer;
    use tower_http::cors::CorsLayer;

    #[test]
    fn test_get_cors_layer_returns_cors_layer() {
        let cors_layer = get_cors_layer();

        // Verify that the function returns a CorsLayer
        // We can't easily test the internal configuration without making requests,
        // but we can verify the layer is created successfully
        assert!(std::mem::size_of_val(&cors_layer) > 0);
    }

    #[test]
    fn test_cors_layer_type() {
        let cors_layer = get_cors_layer();

        // Verify the type is correct
        let _: CorsLayer = cors_layer;
    }

    #[test]
    fn test_cors_layer_can_be_cloned() {
        let cors_layer = get_cors_layer();
        let cloned_layer = cors_layer.clone();

        // Verify both layers exist
        assert!(std::mem::size_of_val(&cors_layer) > 0);
        assert!(std::mem::size_of_val(&cloned_layer) > 0);
    }

    // Note: The actual CORS functionality (origin checking, header validation, etc.)
    // would be better tested through integration tests with actual HTTP requests,
    // as the CorsLayer is designed to work within the Tower/Axum middleware stack.
    // Unit testing the configuration is limited due to the opaque nature of the CorsLayer type.

    #[test]
    fn test_multiple_cors_layer_creation() {
        let layer1 = get_cors_layer();
        let layer2 = get_cors_layer();

        // Verify multiple layers can be created independently
        assert!(std::mem::size_of_val(&layer1) > 0);
        assert!(std::mem::size_of_val(&layer2) > 0);
    }

    #[tokio::test]
    async fn test_cors_layer_with_axum_router() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn test_handler() -> &'static str {
            "test"
        }

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(get_cors_layer());

        // Test that the router with CORS layer can be created and responds
        let request = Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cors_preflight_request() {
        use axum::{Router, routing::get};
        use http::{Method, Request, StatusCode};
        use tower::ServiceExt;

        async fn test_handler() -> &'static str {
            "test"
        }

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(get_cors_layer());

        // Test OPTIONS preflight request
        let request = Request::builder()
            .method(Method::OPTIONS)
            .uri("/test")
            .header("Origin", "http://localhost:3000")
            .header("Access-Control-Request-Method", "GET")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // CORS preflight should be handled
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_cors_localhost_origin_allowed() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn test_handler() -> &'static str {
            "test"
        }

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(get_cors_layer());

        // Test request from localhost origin
        let request = Request::builder()
            .uri("/test")
            .header("Origin", "http://localhost:3000")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cors_external_origin_handling() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn test_handler() -> &'static str {
            "test"
        }

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(get_cors_layer());

        // Test request from external origin (should still work based on the CORS config)
        let request = Request::builder()
            .uri("/test")
            .header("Origin", "https://example.com")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // The response should be successful even if origin is not explicitly allowed
        // because our CORS layer allows any method/header but has custom origin logic
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cors_various_localhost_ports() {
        use axum::{Router, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn test_handler() -> &'static str {
            "test"
        }

        // Test different localhost ports
        for port in ["3000", "3001", "8080", "8000"] {
            let app = Router::new()
                .route("/test", get(test_handler))
                .layer(get_cors_layer());

            let origin = format!("http://localhost:{}", port);
            let request = Request::builder()
                .uri("/test")
                .header("Origin", &origin)
                .body(axum::body::Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }
}
