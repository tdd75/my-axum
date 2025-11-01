use axum::http::StatusCode;
use rust_i18n::t;

use crate::{
    core::{
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    user::{dto::user_dto::UserDTO, service::user_service},
};

pub async fn execute(context: &Context<'_>) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    // Get current user from context
    let current_user = context.user.as_ref().ok_or_else(|| {
        ErrorDTO::new(
            StatusCode::UNAUTHORIZED,
            t!("auth.user_not_authenticated").to_string(),
        )
    })?;

    let user_dto = user_service::model_to_dto(context, current_user).await?;

    Ok(ResponseDTO::new(StatusCode::OK, user_dto))
}
