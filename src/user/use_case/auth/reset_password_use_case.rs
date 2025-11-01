use crate::{
    core::{
        context::Context,
        db::entity::{password_reset_token, user},
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    pkg::password::hash_password_string,
    user::{
        dto::auth_dto::ResetPasswordDTO,
        repository::{password_reset_repository, user_repository},
        service::user_service,
    },
};
use axum::http::StatusCode;
use chrono::Utc;
use sea_orm::entity::*;

pub async fn execute(
    context: &Context<'_>,
    dto: ResetPasswordDTO,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    // Validate email format
    user_service::validate_email_format(&dto.email)?;

    // Validate password
    user_service::validate_password(&dto.new_password)?;

    // Find user by email first
    let user = user_repository::find_by_email(context, &dto.email)
        .await
        .map_err(ErrorDTO::map_internal_error)?
        .ok_or_else(|| {
            ErrorDTO::new(
                StatusCode::BAD_REQUEST,
                "Invalid email or OTP code".to_string(),
            )
        })?;

    // Find reset token by OTP
    let reset_token = password_reset_repository::find_by_token(context, &dto.otp)
        .await
        .map_err(ErrorDTO::map_internal_error)?
        .ok_or_else(|| {
            ErrorDTO::new(
                StatusCode::BAD_REQUEST,
                "Invalid email or OTP code".to_string(),
            )
        })?;

    // Verify OTP belongs to this user
    if reset_token.user_id != user.id {
        tracing::warn!(
            "OTP {} does not belong to user {} (email: {})",
            dto.otp,
            user.id,
            dto.email
        );
        // Increment retry count for security
        let mut reset_token_active: password_reset_token::ActiveModel = reset_token.into();
        reset_token_active.retry_count = Set(reset_token_active.retry_count.unwrap() + 1);
        password_reset_repository::update(context, reset_token_active)
            .await
            .map_err(ErrorDTO::map_internal_error)?;
        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            "Invalid email or OTP code".to_string(),
        ));
    }

    // Check retry count
    const MAX_ATTEMPTS: i32 = 3;
    if reset_token.retry_count >= MAX_ATTEMPTS {
        // Delete token after max retries
        let reset_token_active: password_reset_token::ActiveModel = reset_token.clone().into();
        password_reset_repository::delete(context, reset_token_active)
            .await
            .map_err(ErrorDTO::map_internal_error)?;

        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            format!(
                "Maximum attempts ({}) exceeded. Please request a new OTP code.",
                MAX_ATTEMPTS
            ),
        ));
    }

    // Check if token is expired
    let now = Utc::now().naive_utc();
    if reset_token.expires_at < now {
        // Delete expired token
        let reset_token_active: password_reset_token::ActiveModel = reset_token.into();
        password_reset_repository::delete(context, reset_token_active)
            .await
            .map_err(ErrorDTO::map_internal_error)?;

        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            "OTP code has expired. Please request a new one.".to_string(),
        ));
    }

    // Hash new password
    let hashed_password = hash_password_string(&dto.new_password)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    // Update user's password
    let mut user: user::ActiveModel = user.into();
    user.password = Set(hashed_password);
    user_repository::update(context, user)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    // Store user_id before moving reset_token
    let user_id = reset_token.user_id;

    // Delete the used reset token
    let reset_token_active: password_reset_token::ActiveModel = reset_token.into();
    password_reset_repository::delete(context, reset_token_active)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    // Optionally: Delete all other reset tokens for this user
    password_reset_repository::delete_by_user_id(context, user_id)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    tracing::info!("Password successfully reset for user_id: {}", user_id);

    Ok(ResponseDTO::new(StatusCode::NO_CONTENT, ()))
}
