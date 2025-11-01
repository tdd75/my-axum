use axum::http::StatusCode;
use rust_i18n::t;
use uuid::Uuid;

use crate::{
    config::setting::MessageType,
    core::{
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
        task::{TaskType, publish_task},
    },
    user::{
        dto::avatar_dto::{UploadAvatarDTO, UploadAvatarResponseDTO},
        repository::user_repository,
    },
};

pub async fn execute(
    context: &Context<'_>,
    request: UploadAvatarDTO,
) -> Result<ResponseDTO<UploadAvatarResponseDTO>, ErrorDTO> {
    // Verify user exists
    let user = user_repository::find_by_id(context, request.user_id)
        .await
        .map_err(ErrorDTO::map_internal_error)?
        .ok_or_else(|| ErrorDTO::new(StatusCode::NOT_FOUND, t!("user.not_found").to_string()))?;

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
                "Avatar upload initiated. Connect to ws://your-domain/ws/task/{}/ to track progress.",
                task_id
            ),
        },
    ))
}
