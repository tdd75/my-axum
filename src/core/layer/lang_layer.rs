use axum::{extract::Request, http, http::StatusCode, middleware::Next, response::Response};
use http::HeaderMap;
use rust_i18n::t;
use std::collections::HashMap;

use crate::core::{dto::error_dto::ErrorDTO, translation::language::Language};

#[derive(Clone, Debug)]
pub struct RequestLocale(String);

impl RequestLocale {
    pub fn new(locale: impl Into<String>) -> Self {
        Self(locale.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub async fn lang_middleware(mut req: Request, next: Next) -> Result<Response, ErrorDTO> {
    let locale = get_request_locale(&req)?;
    req.extensions_mut().insert(locale);

    Ok(next.run(req).await)
}

pub fn get_request_locale(req: &Request) -> Result<RequestLocale, ErrorDTO> {
    if let Some(locale) = get_locale_from_query_params(req.uri().query()) {
        return Ok(RequestLocale::new(locale));
    }

    let accept_language = get_accept_language(req.headers())?.unwrap_or("en".to_string());
    let lang = Language::from_accept_language(accept_language.as_str());
    Ok(RequestLocale::new(lang.to_locale()))
}

pub fn get_locale_from_query_params(query: Option<&str>) -> Option<String> {
    let query = query?;
    let params = serde_urlencoded::from_str::<HashMap<String, String>>(query).ok()?;
    let lang = params.get("lang")?;
    Some(Language::from_accept_language(lang).to_locale().to_string())
}

pub fn get_accept_language(header_map: &HeaderMap) -> Result<Option<String>, ErrorDTO> {
    if let Some(lang_header) = header_map.get("Accept-Language") {
        let lang_str = lang_header.to_str().map_err(|_| {
            ErrorDTO::new(
                StatusCode::BAD_REQUEST,
                t!("language.invalid_header", locale = "en").to_string(),
            )
        })?;
        Ok(Some(lang_str.to_string()))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
    use axum::{Router, middleware, routing::get};
    use tower::ServiceExt;

    use super::{
        RequestLocale, get_accept_language, get_locale_from_query_params, get_request_locale,
        lang_middleware,
    };

    #[test]
    fn extracts_accept_language_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept-Language",
            HeaderValue::from_static("vi-VN,vi;q=0.9"),
        );

        let lang = get_accept_language(&headers).unwrap();
        assert_eq!(lang.as_deref(), Some("vi-VN,vi;q=0.9"));
    }

    #[test]
    fn rejects_invalid_accept_language_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept-Language",
            HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap(),
        );

        let error = get_accept_language(&headers).unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn extracts_locale_from_query_param() {
        let locale = get_locale_from_query_params(Some("lang=vi")).unwrap();
        assert_eq!(locale, "vi");
    }

    #[test]
    fn query_param_locale_overrides_accept_language_header() {
        let request = Request::builder()
            .uri("/test?lang=vi")
            .header("Accept-Language", "en-US,en;q=0.9")
            .body(axum::body::Body::empty())
            .unwrap();

        let locale = get_request_locale(&request).unwrap();
        assert_eq!(locale.as_str(), "vi");
    }

    #[tokio::test]
    async fn language_middleware_allows_requests_with_or_without_header() {
        async fn handler(locale: Option<axum::Extension<RequestLocale>>) -> &'static str {
            if let Some(axum::Extension(locale)) = locale {
                let _ = locale.as_str();
            }
            "ok"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(middleware::from_fn(lang_middleware));

        for request in [
            Request::builder()
                .uri("/test")
                .header("Accept-Language", "en-US,en;q=0.9")
                .body(axum::body::Body::empty())
                .unwrap(),
            Request::builder()
                .uri("/test")
                .body(axum::body::Body::empty())
                .unwrap(),
        ] {
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }
}
