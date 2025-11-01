use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadAvatarDTO {
    pub user_id: i32,
    pub file_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadAvatarResponseDTO {
    pub task_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AvatarUploadProgressDTO {
    pub task_id: String,
    pub user_id: i32,
    pub progress: u8,
    pub status: String,
    pub message: Option<String>,
}

impl AvatarUploadProgressDTO {
    pub fn new(task_id: String, user_id: i32, progress: u8, status: &str) -> Self {
        Self {
            task_id,
            user_id,
            progress,
            status: status.to_string(),
            message: None,
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }
}
