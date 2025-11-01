use http::Method;
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

pub fn get_cors_layer() -> CorsLayer {
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
        .allow_origin(AllowOrigin::predicate(|origin, _request_parts| {
            if origin.as_bytes().starts_with(b"http://localhost") {
                return true;
            }

            let allowed_origin: Vec<&str> = vec![];
            allowed_origin
                .iter()
                .any(|allowed_origin| origin == *allowed_origin)
        }))
}
