use axum::http::StatusCode;
use rust_i18n::t;
use sea_orm::Set;

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
    dto: UserUpdateDTO,
    fields: Vec<String>,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    // First, find the existing user
    let existing_user = user_repository::find_by_id(context, id)
        .await
        .map_err(ErrorDTO::map_internal_error)?
        .ok_or_else(|| ErrorDTO::new(StatusCode::NOT_FOUND, t!("user.not_found").to_string()))?;

    // Convert to ActiveModel
    let mut user_active: user::ActiveModel = existing_user.into();

    // Only update fields that were provided
    for field in fields {
        match field.as_str() {
            "email" => {
                if let Some(ref email) = dto.email {
                    // Validate email uniqueness before updating
                    user_service::validate_unique_email(context, email, Some(id)).await?;
                    user_active.email = Set(email.clone());
                }
            }
            "password" => {
                if let Some(ref password) = dto.password {
                    // Validate password strength before hashing
                    password::validate_password_strength(password)
                        .map_err(|e| ErrorDTO::new(StatusCode::BAD_REQUEST, e.to_string()))?;

                    let hashed_password = password::hash_password_string(password)
                        .await
                        .map_err(ErrorDTO::map_internal_error)?;
                    user_active.password = Set(hashed_password);
                }
            }
            "first_name" => user_active.first_name = Set(dto.first_name.clone()),
            "last_name" => user_active.last_name = Set(dto.last_name.clone()),
            "phone" => user_active.phone = Set(dto.phone.clone()),
            _ => {}
        }
    }

    // Update the user
    let user_model = user_repository::update(context, user_active)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    let user_dto = user_service::model_to_dto(context, &user_model).await?;

    Ok(ResponseDTO::new(StatusCode::OK, user_dto))
}
