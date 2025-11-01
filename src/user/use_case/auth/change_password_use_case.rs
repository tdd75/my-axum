use axum::http::StatusCode;
use rust_i18n::t;
use sea_orm::entity::*;

use crate::{
    core::{
        context::Context,
        db::entity::user,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    pkg::password::{hash_password_string, verify_password},
    user::{dto::auth_dto::ChangePasswordDTO, repository::user_repository},
};

pub async fn execute(
    context: &Context<'_>,
    dto: ChangePasswordDTO,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    // Get current user from context
    let current_user = context.user.as_ref().ok_or_else(|| {
        ErrorDTO::new(
            StatusCode::UNAUTHORIZED,
            t!("auth.user_not_authenticated").to_string(),
        )
    })?;
    // Verify old password
    verify_password(&dto.old_password, &current_user.password)
        .await
        .map_err(|_| {
            ErrorDTO::new(
                StatusCode::BAD_REQUEST,
                t!("auth.password_incorrect").to_string(),
            )
        })?;

    // Hash new password
    let hashed_password = hash_password_string(&dto.new_password)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    // Update user password
    let mut user_active_model: user::ActiveModel = current_user.clone().into();
    user_active_model.password = Set(hashed_password);

    user_repository::update(context, user_active_model)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    Ok(ResponseDTO::new(StatusCode::NO_CONTENT, ()))
}
