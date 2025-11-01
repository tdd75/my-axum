use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadAvatarDTO {
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

#[cfg(test)]
mod tests {
    use super::{AvatarUploadProgressDTO, UploadAvatarDTO};

    #[test]
    fn creates_upload_avatar_dto() {
        let dto = UploadAvatarDTO {
            file_name: "avatar.jpg".to_string(),
        };

        assert_eq!(dto.file_name, "avatar.jpg");
    }

    #[test]
    fn builds_progress_dto_with_message() {
        let dto = AvatarUploadProgressDTO::new("task-1".to_string(), 1, 50, "uploading")
            .with_message("Halfway there");

        assert_eq!(dto.task_id, "task-1");
        assert_eq!(dto.progress, 50);
        assert_eq!(dto.message.as_deref(), Some("Halfway there"));
    }

    #[test]
    fn serializes_progress_dto() {
        let dto = AvatarUploadProgressDTO::new("task-2".to_string(), 2, 100, "completed");
        let json = serde_json::to_string(&dto).unwrap();

        assert!(json.contains("task-2"));
        assert!(json.contains("completed"));
    }
}
