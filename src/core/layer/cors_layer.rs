use crate::config::setting::Setting;
use crate::pkg::cors::matches_origin_pattern;
use http::Method;
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

pub fn get_cors_layer() -> CorsLayer {
    let setting = Setting::new();
    let allowed_origins = setting.allowed_origins.clone();

    CorsLayer::new()
        .allow_headers(AllowHeaders::list(vec![
            AUTHORIZATION,
            CONTENT_TYPE,
            ACCEPT,
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
