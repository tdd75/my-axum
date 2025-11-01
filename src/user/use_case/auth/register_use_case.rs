use crate::{
    config::setting::MessageType,
    core::{
        context::Context,
        db::entity::user,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
        task::{TaskType, publish_task},
    },
    pkg::password::hash_password_string,
    user::{
        dto::auth_dto::{RegisterDTO, TokenPairDTO},
        repository::user_repository,
        service::{auth_service, user_service},
    },
};
use axum::http::{HeaderMap, StatusCode};
use sea_orm::entity::*;

pub async fn execute(
    context: &Context<'_>,
    dto: RegisterDTO,
    headers: HeaderMap,
) -> Result<ResponseDTO<TokenPairDTO>, ErrorDTO> {
    // Validate email format
    user_service::validate_email_format(&dto.email)?;

    // Validate password
    user_service::validate_password(&dto.password)?;

    // Check email uniqueness
    user_service::validate_unique_email(context, &dto.email, None).await?;

    let hashed_password = hash_password_string(&dto.password)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    let user = user::ActiveModel {
        email: Set(dto.email),
        password: Set(hashed_password),
        first_name: Set(dto.first_name),
        last_name: Set(dto.last_name),
        phone: Set(dto.phone),
        ..Default::default()
    };
    let user = user_repository::create(context, user).await.unwrap();

    let (access, refresh) = auth_service::generate_token_pair(user.id).await?;

    // Save refresh token to database
    auth_service::create_refresh_token_record(context, user.id, &refresh, &headers).await?;

    // Send welcome email
    send_welcome_email(context, &user).await?;

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

async fn send_welcome_email(context: &Context<'_>, user: &user::Model) -> Result<(), ErrorDTO> {
    let user_id = user.id;
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
                "Message producer not available in context. Skipping welcome email task publishing."
            );
        }
    }

    Ok(())
}
