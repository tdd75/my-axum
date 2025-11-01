use axum::http::StatusCode;
use rust_i18n::t;
use sea_orm::Set;

use crate::{
    core::{
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    user::{
        dto::auth_dto::{ProfileDTO, UpdateProfileDTO},
        repository::user_repository,
    },
};

pub async fn execute(
    context: &Context<'_>,
    dto: UpdateProfileDTO,
    fields: Vec<String>,
) -> Result<ResponseDTO<ProfileDTO>, ErrorDTO> {
    // Get current user from context
    let current_user = context.user.as_ref().ok_or_else(|| {
        ErrorDTO::new(
            StatusCode::UNAUTHORIZED,
            t!("auth.user_not_authenticated").to_string(),
        )
    })?;

    // Convert user to active model
    let mut user: crate::core::db::entity::user::ActiveModel = current_user.clone().into();

    // Only update fields that were provided
    for field in fields {
        match field.as_str() {
            "first_name" => user.first_name = Set(dto.first_name.clone()),
            "last_name" => user.last_name = Set(dto.last_name.clone()),
            "phone" => user.phone = Set(dto.phone.clone()),
            _ => {}
        }
    }

    // Update the user
    let updated_user = user_repository::update(context, user)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    // Convert to ProfileDTO
    let profile_dto = ProfileDTO::from(updated_user);

    Ok(ResponseDTO::new(StatusCode::OK, profile_dto))
}
