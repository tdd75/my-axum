use axum::body::Body;
use http::{Request, Response};
use std::time::Duration;
use tower_http::trace;
use tracing::{Level, Span};

#[allow(clippy::type_complexity)]
pub fn get_trace_layer() -> trace::TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    impl Fn(&Request<Body>) -> Span + Clone,
    trace::DefaultOnRequest,
    impl Fn(&Response<Body>, Duration, &Span) + Clone,
    trace::DefaultOnBodyChunk,
    trace::DefaultOnEos,
    trace::DefaultOnFailure,
> {
    trace::TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Body>| {
            tracing::span!(
                Level::INFO,
                "request",
                method = display(request.method()),
                uri = display(request.uri()),
                version = debug(request.version()),
                request_id = display(uuid::Uuid::new_v4()),
            )
        })
        .on_response(
            |response: &Response<Body>, latency: Duration, _span: &Span| {
                tracing::info!(
                    status = response.status().as_u16(),
                    latency = ?latency,
                    "response"
                );
            },
        )
}

#[cfg(test)]
mod tests {
    use axum::{Json, Router, routing::get};
    use http::{Method, Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    use super::get_trace_layer;

    #[test]
    fn creates_trace_layer() {
        let layer = get_trace_layer();
        assert!(std::mem::size_of_val(&layer) > 0);
    }

    #[tokio::test]
    async fn traces_success_and_not_found_responses() {
        async fn handler() -> &'static str {
            "ok"
        }

        let app = Router::new()
            .route("/exists", get(handler))
            .layer(get_trace_layer());

        let ok = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/exists")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok.status(), StatusCode::OK);

        let not_found = app
            .oneshot(
                Request::builder()
                    .uri("/missing")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(not_found.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn traces_different_methods_and_payloads() {
        async fn get_handler() -> &'static str {
            "get"
        }

        async fn post_handler(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
            Json(payload)
        }

        let app = Router::new()
            .route("/test", get(get_handler).post(post_handler))
            .layer(get_trace_layer());

        let get_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/test")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        let post_response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/test")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(axum::body::Body::from(json!({ "ok": true }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(post_response.status(), StatusCode::OK);
    }
}
