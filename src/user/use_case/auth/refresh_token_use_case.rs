use axum::http::{HeaderMap, StatusCode};
use rust_i18n::t;

use crate::{
    config::setting::Setting,
    core::{
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    pkg::jwt::decode_token,
    user::{
        dto::auth_dto::{RefreshTokenDTO, TokenPairDTO},
        repository::{refresh_token_repository, user_repository},
        service::auth_service::{self, TokenType},
    },
};

pub async fn execute(
    context: &Context<'_>,
    dto: RefreshTokenDTO,
    headers: HeaderMap,
) -> Result<ResponseDTO<TokenPairDTO>, ErrorDTO> {
    // Priority 1: Check if token is provided in the request body
    let refresh_token = match dto.refresh_token {
        Some(token) => token,
        None => {
            // Priority 2: Try to get token from header or cookie
            auth_service::extract_token_from_header_or_cookie(&headers, TokenType::Refresh).await?
        }
    };
    let user_id = validate_jwt_token(&refresh_token)?;

    // Check if user exists
    let user = user_repository::find_by_id(context, user_id)
        .await
        .map_err(ErrorDTO::map_internal_error)?
        .ok_or_else(|| {
            ErrorDTO::new(
                StatusCode::UNAUTHORIZED,
                t!("auth.user_not_found").to_string(),
            )
        })?;

    // Check if refresh token exists and is valid in database
    refresh_token_repository::find_by_user_and_token(context, user.id, &refresh_token)
        .await
        .map_err(ErrorDTO::map_internal_error)?
        .ok_or_else(|| {
            ErrorDTO::new(
                StatusCode::UNAUTHORIZED,
                t!("auth.refresh_token_invalid").to_string(),
            )
        })?;

    // Delete old refresh token
    refresh_token_repository::delete_by_token(context, &refresh_token)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    // Generate new token pair
    let (new_access, new_refresh) = auth_service::generate_token_pair(user.id).await?;

    // Save new refresh token to database
    auth_service::create_refresh_token_record(context, user.id, &new_refresh, &headers).await?;

    let response_data = TokenPairDTO {
        access: new_access.clone(),
        refresh: new_refresh.clone(),
    };

    let mut response_headers = HeaderMap::new();
    auth_service::set_auth_cookies(&mut response_headers, &new_access, &new_refresh);

    Ok(ResponseDTO::with_headers(
        StatusCode::OK,
        response_data,
        response_headers,
    ))
}

fn validate_jwt_token(refresh_token: &str) -> Result<i32, ErrorDTO> {
    let setting = Setting::new();
    let claims = decode_token(refresh_token, &setting.jwt_secret).map_err(|_| {
        ErrorDTO::new(
            StatusCode::UNAUTHORIZED,
            t!("auth.refresh_token_invalid").to_string(),
        )
    })?;

    Ok(claims.sub)
}
