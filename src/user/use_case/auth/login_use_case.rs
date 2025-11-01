use axum::http::{HeaderMap, StatusCode};
use rust_i18n::t;

use crate::{
    core::{
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    pkg::password::verify_password,
    user::{
        dto::auth_dto::{LoginDTO, TokenPairDTO},
        repository::user_repository,
        service::auth_service,
    },
};

pub async fn execute(
    context: &Context<'_>,
    dto: LoginDTO,
    headers: HeaderMap,
) -> Result<ResponseDTO<TokenPairDTO>, ErrorDTO> {
    let user = user_repository::find_by_email(context, &dto.email)
        .await
        .map_err(ErrorDTO::map_internal_error)?
        .ok_or_else(|| {
            ErrorDTO::new(
                StatusCode::UNAUTHORIZED,
                t!("auth.email_not_registered").to_string(),
            )
        })?;

    verify_password(&dto.password, &user.password)
        .await
        .map_err(|_| {
            ErrorDTO::new(
                StatusCode::UNAUTHORIZED,
                t!("auth.password_incorrect").to_string(),
            )
        })?;

    let (access, refresh) = auth_service::generate_token_pair(user.id).await?;

    // Save refresh token to database
    auth_service::create_refresh_token_record(context, user.id, &refresh, &headers).await?;

    let response_data = TokenPairDTO {
        access: access.clone(),
        refresh: refresh.clone(),
    };

    let mut response_headers = HeaderMap::new();
    auth_service::set_auth_cookies(&mut response_headers, &access, &refresh);

    Ok(ResponseDTO::with_headers(
        StatusCode::OK,
        response_data,
        response_headers,
    ))
}
