use regex::Regex;
use rust_i18n::t;
use std::collections::HashMap;

use crate::{
    core::{
        context::Context,
        db::entity::user::{self},
        dto::error_dto::ErrorDTO,
    },
    user::{
        dto::user_dto::{UserDTO, UserWithRelations},
        repository::user_repository::{self, UserSearchParams},
    },
};
use axum::http::StatusCode;

pub async fn read(context: &Context<'_>, user_id: i32) -> Result<user::Model, ErrorDTO> {
    let user = user_repository::find_by_id(context, user_id)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    user.ok_or_else(|| {
        ErrorDTO::new(
            StatusCode::NOT_FOUND,
            t!("user.not_found_with_id", id = user_id).to_string(),
        )
    })
}

pub async fn build_user_map(
    context: &Context<'_>,
    user_ids: &[i32],
) -> Result<HashMap<i32, user::Model>, ErrorDTO> {
    let users = user_repository::search(
        context,
        &UserSearchParams {
            ids: Some(user_ids),
            ..Default::default()
        },
    )
    .await
    .map_err(ErrorDTO::map_internal_error)?;

    let user_map: HashMap<i32, user::Model> =
        users.into_iter().map(|user| (user.id, user)).collect();

    Ok(user_map)
}

// ------------------------------------------------
// Serialization
// ------------------------------------------------

pub async fn models_to_dtos(
    context: &Context<'_>,
    users: &[user::Model],
) -> Result<Vec<UserDTO>, ErrorDTO> {
    let created_user_ids: Vec<i32> = users.iter().filter_map(|u| u.created_user_id).collect();
    let updated_user_ids: Vec<i32> = users.iter().filter_map(|u| u.updated_user_id).collect();
    let user_map = build_user_map(context, &[created_user_ids, updated_user_ids].concat()).await?;

    let user_dtos: Vec<UserDTO> = users
        .iter()
        .map(|user| {
            let created_user = match user.created_user_id {
                Some(id) => user_map.get(&id).cloned(),
                None => None,
            };
            let updated_user = match user.updated_user_id {
                Some(id) => user_map.get(&id).cloned(),
                None => None,
            };

            UserWithRelations {
                model: user.clone(),
                created_user,
                updated_user,
            }
            .into()
        })
        .collect();

    Ok(user_dtos)
}

pub async fn model_to_dto(context: &Context<'_>, user: &user::Model) -> Result<UserDTO, ErrorDTO> {
    models_to_dtos(context, std::slice::from_ref(user))
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| {
            ErrorDTO::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to convert user model to DTO".to_string(),
            )
        })
}

// ------------------------------------------------
// Validation
// ------------------------------------------------

pub async fn validate_unique_email(
    context: &Context<'_>,
    email: &str,
    exclude_id: Option<i32>,
) -> Result<(), ErrorDTO> {
    let existing_user = user_repository::find_by_email(context, email)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    if let Some(existing_user) = existing_user
        && (exclude_id.is_none() || Some(existing_user.id) != exclude_id)
    {
        return Err(ErrorDTO::new(
            StatusCode::CONFLICT,
            t!("user.email_already_in_use").to_string(),
        ));
    }

    Ok(())
}

pub fn validate_email_format(email: &str) -> Result<(), ErrorDTO> {
    if email.trim().is_empty() {
        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            "Email is required".to_string(),
        ));
    }

    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(ErrorDTO::map_internal_error)?;
    if !email_regex.is_match(email) {
        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            "Invalid email format".to_string(),
        ));
    }

    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), ErrorDTO> {
    if password.trim().is_empty() {
        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            "Password is required".to_string(),
        ));
    }

    Ok(())
}
