use axum::http::StatusCode;
use sea_orm::entity::*;

use crate::{
    config::setting::MessageType,
    core::{
        r#async::{TaskType, publish_task},
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    pkg::password,
    user::{
        dto::user_dto::{UserCreateDTO, UserDTO},
        entity::{sea_orm_active_enums::UserRole, user},
        repository::user_repository,
        service::user_service,
    },
};

pub async fn execute(
    context: &Context,
    dto: UserCreateDTO,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    // Validate email format first
    user_service::validate_email_format(&dto.email, &context.locale)?;

    // Validate password strength
    user_service::validate_password(&dto.password, &context.locale)?;

    // Check for email uniqueness
    user_service::validate_unique_email(context, &dto.email, None).await?;

    // Hash the password before storing
    let hashed_password = password::hash_password_string(&dto.password)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    let user = user::ActiveModel {
        email: Set(dto.email),
        password: Set(hashed_password),
        role: Set(UserRole::User),
        first_name: Set(dto.first_name),
        last_name: Set(dto.last_name),
        phone: Set(dto.phone),
        ..Default::default()
    };

    let user_model = user_repository::create(context, user)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    let user_dto = user_service::model_to_dto(context, &user_model).await?;

    // Send welcome email
    send_welcome_email(context, user_model.id).await?;

    Ok(ResponseDTO::new(StatusCode::CREATED, user_dto))
}

async fn send_welcome_email(context: &Context, user_id: i32) -> Result<(), ErrorDTO> {
    match &context.producer {
        Some(producer) => {
            if let Err(e) = publish_task(
                producer.as_ref().as_ref(),
                TaskType::ProcessUserRegistration { user_id },
                Some(MessageType::Emails.as_ref()),
            )
            .await
            {
                tracing::error!("Failed to publish welcome email task: {}", e);
            }
        }
        None => {
            tracing::warn!(
                "Message producer is not available in context. Skipping welcome email task publishing."
            );
        }
    }

    Ok(())
}
