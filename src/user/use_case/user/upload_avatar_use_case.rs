use axum::http::StatusCode;
use rust_i18n::t;
use uuid::Uuid;

use crate::{
    config::setting::MessageType,
    core::{
        r#async::{TaskType, publish_task},
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    user::{
        dto::avatar_dto::{UploadAvatarDTO, UploadAvatarResponseDTO},
        repository::user_repository,
    },
};

pub async fn execute(
    context: &Context,
    request: UploadAvatarDTO,
    locale: &str,
) -> Result<ResponseDTO<UploadAvatarResponseDTO>, ErrorDTO> {
    let current_user = context.user.as_ref().ok_or_else(|| {
        ErrorDTO::new(
            StatusCode::UNAUTHORIZED,
            t!("auth.user_not_authenticated", locale = &context.locale).to_string(),
        )
    })?;

    let user = user_repository::find_by_id(context, current_user.id)
        .await
        .map_err(ErrorDTO::map_internal_error)?
        .ok_or_else(|| {
            ErrorDTO::new(
                StatusCode::NOT_FOUND,
                t!("user.not_found", locale = &context.locale).to_string(),
            )
        })?;

    // Generate random task_id (UUID v4)
    let task_id = Uuid::new_v4().to_string();

    match &context.producer {
        Some(producer) => {
            publish_task(
                producer.as_ref().as_ref(),
                TaskType::ProcessAvatarUpload {
                    task_id: task_id.clone(),
                    user_id: user.id,
                    file_name: request.file_name.clone(),
                    locale: locale.to_string(),
                },
                Some(MessageType::Tasks.as_ref()),
            )
            .await
            .map_err(|e| {
                ErrorDTO::map_internal_error(anyhow::anyhow!(
                    "Failed to publish upload task: {}",
                    e
                ))
            })?;
        }
        None => {
            return Err(ErrorDTO::map_internal_error(anyhow::anyhow!(
                "Producer not available"
            )));
        }
    }

    Ok(ResponseDTO::new(
        StatusCode::ACCEPTED,
        UploadAvatarResponseDTO {
            task_id: task_id.clone(),
            message: format!(
                "Avatar upload initiated. Connect to ws://your-domain/ws/v1/task/{}/ to track progress.",
                task_id
            ),
        },
    ))
}
