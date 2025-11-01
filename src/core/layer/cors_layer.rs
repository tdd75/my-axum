use crate::config::setting::Setting;
use crate::pkg::cors::matches_origin_pattern;
use http::Method;
use http::header::{ACCEPT, ACCEPT_LANGUAGE, AUTHORIZATION, CONTENT_TYPE};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

pub fn get_cors_layer() -> CorsLayer {
    let setting = Setting::new();
    let allowed_origins = setting.allowed_origins.clone();

    CorsLayer::new()
        .allow_headers(AllowHeaders::list(vec![
            AUTHORIZATION,
            CONTENT_TYPE,
            ACCEPT,
            ACCEPT_LANGUAGE,
        ]))
        .allow_methods(AllowMethods::list(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ]))
        .allow_credentials(true)
        .allow_origin(AllowOrigin::predicate(move |origin, _request_parts| {
            // Allow localhost in development
            if origin.as_bytes().starts_with(b"http://localhost") {
                return true;
            }

            // Check against configured allowed origins with wildcard support
            let origin_str = origin.to_str().unwrap_or("");
            allowed_origins
                .iter()
                .any(|allowed| matches_origin_pattern(origin_str, allowed))
        }))
}

#[cfg(test)]
mod tests {
    use axum::{Router, routing::get};
    use http::header::ACCESS_CONTROL_ALLOW_HEADERS;
    use http::{Method, Request, StatusCode};
    use tower::ServiceExt;
    use tower_http::cors::CorsLayer;

    use super::get_cors_layer;

    #[test]
    fn builds_a_cors_layer() {
        let layer = get_cors_layer();
        let _: CorsLayer = layer;
    }

    #[tokio::test]
    async fn allows_simple_requests_through_router() {
        async fn handler() -> &'static str {
            "ok"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(get_cors_layer());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "http://localhost:3000")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn handles_preflight_requests() {
        async fn handler() -> &'static str {
            "ok"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(get_cors_layer());
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/test")
                    .header("Origin", "http://localhost:3000")
                    .header("Access-Control-Request-Method", "POST")
                    .header(
                        "Access-Control-Request-Headers",
                        "content-type,accept-language",
                    )
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NO_CONTENT);
        let allow_headers = response
            .headers()
            .get(ACCESS_CONTROL_ALLOW_HEADERS)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        assert!(allow_headers.contains("accept-language"));
    }
}
