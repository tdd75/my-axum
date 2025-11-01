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
