use axum::http::{HeaderMap, StatusCode};

use crate::{
    core::{
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    user::{repository::refresh_token_repository, service::auth_service},
};

pub async fn execute(
    context: &Context<'_>,
    headers: HeaderMap,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    // Try to get refresh token from cookie to delete it
    if let Some(refresh_token) = auth_service::get_token_from_cookies(&headers, "refresh_token") {
        // Delete refresh token from database if found
        refresh_token_repository::delete_by_token(context, &refresh_token)
            .await
            .map_err(ErrorDTO::map_internal_error)?;
    }

    // Create response headers to clear cookies
    let mut response_headers = HeaderMap::new();
    clear_auth_cookies(&mut response_headers);

    Ok(ResponseDTO::with_headers(
        StatusCode::NO_CONTENT,
        (),
        response_headers,
    ))
}

fn clear_auth_cookies(headers: &mut HeaderMap) {
    use axum::http::header::HeaderValue;

    // Clear access token cookie
    if let Ok(access_clear) =
        HeaderValue::from_str("access_token=; Max-Age=0; HttpOnly; SameSite=Strict; Path=/")
    {
        headers.append("set-cookie", access_clear);
    }

    // Clear refresh token cookie
    if let Ok(refresh_clear) =
        HeaderValue::from_str("refresh_token=; Max-Age=0; HttpOnly; SameSite=Strict; Path=/")
    {
        headers.append("set-cookie", refresh_clear);
    }
}
