use serde::{Deserialize, Serialize};

/// Application-specific task types that can be processed by the worker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskType {
    /// Send email notification
    SendEmail {
        to: String,
        subject: String,
        text_body: Option<String>,
        html_body: Option<String>,
    },

    /// Clean up expired data
    CleanupExpiredToken,

    /// Process user registration
    ProcessUserRegistration { user_id: i32 },

    /// Process avatar upload with progress tracking
    ProcessAvatarUpload {
        task_id: String,
        user_id: i32,
        file_name: String,
    },
}
