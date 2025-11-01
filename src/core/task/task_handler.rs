use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    pkg::messaging::MessageProducer,
    pkg::smtp::SmtpClient,
    user::task::{auth_task, user_task},
};

use super::{TaskEvent, TaskType};
use crate::pkg::messaging::TaskHandler;

/// Concrete task handler implementation for processing different types of tasks
/// This handler does not contain business logic, it only delegates to tasks in modules
pub struct ConcreteTaskHandler {
    db: DatabaseConnection,
    producer: Arc<Box<dyn MessageProducer>>,
    smtp_client: Option<SmtpClient>,
    redis_url: String,
}

impl ConcreteTaskHandler {
    pub fn new(
        db: DatabaseConnection,
        producer: Arc<Box<dyn MessageProducer>>,
        smtp_client: Option<SmtpClient>,
        redis_url: String,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            db,
            producer,
            smtp_client,
            redis_url,
        })
    }
}

#[async_trait]
impl TaskHandler<TaskType> for ConcreteTaskHandler {
    /// Process a task event
    async fn handle_task(&self, event: &TaskEvent) -> anyhow::Result<()> {
        info!(
            "Processing task {} of type {:?}",
            event.id,
            std::mem::discriminant(&event.task)
        );

        let result = match &event.task {
            TaskType::SendEmail {
                to,
                subject,
                text_body,
                html_body,
            } => {
                // Send email
                self.smtp_client
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("SMTP client not configured"))?
                    .send_multipart_mail(
                        to,
                        subject,
                        text_body.clone().unwrap_or_default(),
                        html_body.clone().unwrap_or_default(),
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to send email to {}: {}", to, e))
            }

            TaskType::CleanupExpiredToken => auth_task::clean_expired_tokens(&self.db).await,

            TaskType::ProcessUserRegistration { user_id } => {
                user_task::send_welcome_email(&self.db, self.producer.as_ref().as_ref(), *user_id)
                    .await
            }

            TaskType::ProcessAvatarUpload {
                task_id,
                user_id,
                file_name,
            } => {
                user_task::process_avatar_upload(
                    &self.db,
                    self.producer.as_ref().as_ref(),
                    &self.redis_url,
                    task_id.clone(),
                    *user_id,
                    file_name.clone(),
                )
                .await
            }
        };

        match result {
            Ok(_) => {
                info!("Successfully processed task {}", event.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to process task {}: {:?}", event.id, e);
                Err(e)
            }
        }
    }
}
