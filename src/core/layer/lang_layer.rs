use axum::{extract::Request, http, http::StatusCode, middleware::Next, response::Response};
use http::HeaderMap;

use crate::core::{dto::error_dto::ErrorDTO, translation::language::Language};

pub async fn lang_middleware(req: Request, next: Next) -> Result<Response, ErrorDTO> {
    let accept_language = get_accept_language(req.headers())
        .await?
        .unwrap_or("en".to_string());
    let lang = Language::from_accept_language(accept_language.as_str());
    rust_i18n::set_locale(lang.to_locale());

    Ok(next.run(req).await)
}

pub async fn get_accept_language(header_map: &HeaderMap) -> Result<Option<String>, ErrorDTO> {
    if let Some(lang_header) = header_map.get("Accept-Language") {
        let lang_str = lang_header.to_str().map_err(|_| {
            ErrorDTO::new(
                StatusCode::BAD_REQUEST,
                rust_i18n::t!("language.invalid_header").to_string(),
            )
        })?;
        Ok(Some(lang_str.to_string()))
    } else {
        Ok(None)
    }
}
