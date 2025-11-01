use axum::http::StatusCode;
use rust_i18n::t;

use crate::{
    core::{
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    user::dto::auth_dto::ProfileDTO,
};

pub async fn execute(context: &Context<'_>) -> Result<ResponseDTO<ProfileDTO>, ErrorDTO> {
    // Get current user from context
    let current_user = context.user.as_ref().ok_or_else(|| {
        ErrorDTO::new(
            StatusCode::UNAUTHORIZED,
            t!("auth.user_not_authenticated").to_string(),
        )
    })?;

    let profile_dto = ProfileDTO::from(current_user.clone());

    Ok(ResponseDTO::new(StatusCode::OK, profile_dto))
}
