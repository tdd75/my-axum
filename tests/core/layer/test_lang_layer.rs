#[cfg(test)]
mod lang_layer_tests {
    use axum::http::{HeaderMap, HeaderValue, StatusCode};
    use my_axum::core::layer::lang_layer::{get_accept_language, lang_middleware};

    #[tokio::test]
    async fn test_get_accept_language_with_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept-Language",
            HeaderValue::from_static("en-US,en;q=0.9"),
        );

        let result = get_accept_language(&headers).await.unwrap();
        assert_eq!(result, Some("en-US,en;q=0.9".to_string()));
    }

    #[tokio::test]
    async fn test_get_accept_language_without_header() {
        let headers = HeaderMap::new();

        let result = get_accept_language(&headers).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_get_accept_language_invalid_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept-Language",
            HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap(),
        );

        let result = get_accept_language(&headers).await;
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.status, StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn test_get_accept_language_empty_header() {
        let mut headers = HeaderMap::new();
        headers.insert("Accept-Language", HeaderValue::from_static(""));

        let result = get_accept_language(&headers).await.unwrap();
        assert_eq!(result, Some("".to_string()));
    }

    #[tokio::test]
    async fn test_get_accept_language_vietnamese() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept-Language",
            HeaderValue::from_static("vi-VN,vi;q=0.9,en;q=0.8"),
        );

        let result = get_accept_language(&headers).await.unwrap();
        assert_eq!(result, Some("vi-VN,vi;q=0.9,en;q=0.8".to_string()));
    }

    #[tokio::test]
    async fn test_get_accept_language_complex_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept-Language",
            HeaderValue::from_static("fr;q=0.9,en;q=0.8,vi;q=0.7,de;q=0.6"),
        );

        let result = get_accept_language(&headers).await.unwrap();
        assert_eq!(
            result,
            Some("fr;q=0.9,en;q=0.8,vi;q=0.7,de;q=0.6".to_string())
        );
    }

    #[tokio::test]
    async fn test_lang_middleware_basic() {
        use axum::{Router, middleware, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn test_handler() -> &'static str {
            "test"
        }

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(lang_middleware));

        let request = Request::builder()
            .uri("/test")
            .header("Accept-Language", "en-US")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_lang_middleware_without_header() {
        use axum::{Router, middleware, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn test_handler() -> &'static str {
            "test"
        }

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(lang_middleware));

        let request = Request::builder()
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_lang_middleware_vietnamese() {
        use axum::{Router, middleware, routing::get};
        use http::{Request, StatusCode};
        use tower::ServiceExt;

        async fn test_handler() -> &'static str {
            "test"
        }

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(lang_middleware));

        let request = Request::builder()
            .uri("/test")
            .header("Accept-Language", "vi-VN,vi;q=0.9")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
