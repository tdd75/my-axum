use axum::http::StatusCode;
use rust_i18n::t;
use sea_orm::entity::*;

use crate::{
    core::{
        context::Context,
        db::entity::user,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    pkg::password,
    user::{
        dto::user_dto::{UserDTO, UserUpdateDTO},
        repository::user_repository,
        service::user_service,
    },
};

pub async fn execute(
    context: &Context<'_>,
    id: i32,
    request: UserUpdateDTO,
    fields: Vec<String>,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    // First, find the existing user
    let existing_user = user_repository::find_by_id(context, id)
        .await
        .map_err(ErrorDTO::map_internal_error)?
        .ok_or_else(|| ErrorDTO::new(StatusCode::NOT_FOUND, t!("user.not_found").to_string()))?;

    // Convert to ActiveModel
    let mut user_active: user::ActiveModel = existing_user.into();

    // Update fields only if they are in the update_keys
    if fields.contains(&"email".to_string())
        && let Some(email) = &request.email
    {
        // Validate email uniqueness before updating
        user_service::validate_unique_email(context, email, Some(id)).await?;
        user_active.email = Set(email.clone());
    }

    if fields.contains(&"password".to_string())
        && let Some(password) = request.password
    {
        // Validate password strength before hashing
        password::validate_password_strength(&password)
            .map_err(|e| ErrorDTO::new(StatusCode::BAD_REQUEST, e.to_string()))?;

        let hashed_password = password::hash_password_string(&password)
            .await
            .map_err(ErrorDTO::map_internal_error)?;
        user_active.password = Set(hashed_password);
    }

    if fields.contains(&"first_name".to_string())
        && let Some(first_name) = request.first_name
    {
        user_active.first_name = Set(Some(first_name));
    }

    if fields.contains(&"last_name".to_string())
        && let Some(last_name) = request.last_name
    {
        user_active.last_name = Set(Some(last_name));
    }

    if fields.contains(&"phone".to_string())
        && let Some(phone) = request.phone
    {
        user_active.phone = Set(Some(phone));
    }

    let user_model = user_repository::update(context, user_active)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    let user_dto = user_service::model_to_dto(context, &user_model).await?;

    Ok(ResponseDTO::new(StatusCode::OK, user_dto))
}
