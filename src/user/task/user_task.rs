use chrono::Datelike;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

use crate::{
    config::setting::{MessageType, Setting},
    core::{
        context::Context,
        task::{TaskType, publish_task},
        template::engine::render_email_template,
    },
    pkg::broadcast::websocket::BroadcastMessage,
    pkg::cache::cache_task_status,
    pkg::messaging::MessageProducer,
    user::dto::avatar_dto::AvatarUploadProgressDTO,
    user::repository::user_repository,
};

pub async fn send_welcome_email(
    db: &DatabaseConnection,
    producer: &dyn MessageProducer,
    user_id: i32,
) -> anyhow::Result<()> {
    tracing::info!("Sending welcome email to user id: {}", user_id);

    // Fetch user from database
    let user = db
        .transaction::<_, _, anyhow::Error>(|txn| {
            Box::pin(async move {
                let context = Context {
                    txn,
                    user: None,
                    producer: None,
                };

                let user = user_repository::find_by_id(&context, user_id)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to find user: {}", e))?
                    .ok_or_else(|| anyhow::anyhow!("User not found"))?;

                Ok(user)
            })
        })
        .await?;

    tracing::info!("Publishing welcome email task for user: {}", user.email);

    let setting = Setting::new();

    // Prepare template variables
    let mut variables = HashMap::new();
    variables.insert("app_name".to_string(), "My Axum App".to_string());
    variables.insert("app_url".to_string(), setting.app_url.clone());
    variables.insert("email".to_string(), user.email.clone());
    variables.insert(
        "first_name".to_string(),
        user.first_name.clone().unwrap_or_default(),
    );
    variables.insert(
        "last_name".to_string(),
        user.last_name.clone().unwrap_or_default(),
    );
    variables.insert("phone".to_string(), user.phone.clone().unwrap_or_default());
    variables.insert("year".to_string(), chrono::Utc::now().year().to_string());

    // Render template
    let html_body = render_email_template("email/welcome.html", variables)?;

    // Publish email task to worker instead of sending directly
    publish_task(
        producer,
        TaskType::SendEmail {
            to: user.email.clone(),
            subject: "Welcome to My Axum App!".to_string(),
            text_body: None,
            html_body: Some(html_body),
        },
        Some(MessageType::Emails.as_ref()),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to publish email task: {}", e))?;

    tracing::info!(
        "✓ Welcome email task published to worker for: {}",
        user.email
    );

    Ok(())
}

pub async fn process_avatar_upload(
    db: &DatabaseConnection,
    producer: &dyn MessageProducer,
    redis_url: &str,
    task_id: String,
    user_id: i32,
    file_name: String,
) -> anyhow::Result<()> {
    tracing::info!(
        "Processing avatar upload for task_id: {}, user_id: {}, file_name: {}",
        task_id,
        user_id,
        file_name
    );

    // Verify user exists
    db.transaction::<_, _, anyhow::Error>(|txn| {
        Box::pin(async move {
            let context = Context {
                txn,
                user: None,
                producer: None,
            };

            user_repository::find_by_id(&context, user_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to find user: {}", e))?
                .ok_or_else(|| anyhow::anyhow!("User not found"))?;

            Ok(())
        })
    })
    .await?;

    // Simulate upload progress with sleep
    let stages = vec![
        (10, "Validating file...", 100),
        (25, "Preparing upload...", 200),
        (40, "Processing image...", 400),
        (60, "Optimizing...", 500),
        (80, "Finalizing...", 300),
        (100, "Upload complete!", 200),
    ];

    for (progress, message, delay_ms) in stages {
        sleep(Duration::from_millis(delay_ms)).await;

        let progress_dto =
            AvatarUploadProgressDTO::new(task_id.clone(), user_id, progress, "processing")
                .with_message(message);

        let broadcast_msg = BroadcastMessage {
            event_type: "avatar_upload_progress".to_string(),
            data: serde_json::to_value(&progress_dto)
                .map_err(|e| anyhow::anyhow!("Failed to serialize progress: {}", e))?,
        };

        // Cache task status in Redis (for late WebSocket connections)
        if let Err(e) = cache_task_status(redis_url, &task_id, &broadcast_msg.data).await {
            tracing::warn!("Failed to cache task status in Redis: {}", e);
        }

        // Publish progress to broadcasts queue (will be picked up by forwarder and sent to WebSocket)
        let msg_json = serde_json::to_string(&broadcast_msg)
            .map_err(|e| anyhow::anyhow!("Failed to serialize broadcast message: {}", e))?;

        producer
            .publish_event_json(&msg_json, Some("broadcasts"))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to publish progress: {}", e))?;

        tracing::info!(
            "Avatar upload progress for task {}: {}% - {}",
            task_id,
            progress,
            message
        );
    }

    // Final success message
    let final_progress = AvatarUploadProgressDTO::new(task_id.clone(), user_id, 100, "completed")
        .with_message(&format!("Avatar '{}' uploaded successfully!", file_name));

    let final_msg = BroadcastMessage {
        event_type: "avatar_upload_complete".to_string(),
        data: serde_json::to_value(&final_progress)
            .map_err(|e| anyhow::anyhow!("Failed to serialize final progress: {}", e))?,
    };

    // Cache final task status in Redis (for late WebSocket connections)
    if let Err(e) = cache_task_status(redis_url, &task_id, &final_msg.data).await {
        tracing::warn!("Failed to cache final task status in Redis: {}", e);
    }

    // Publish final message to broadcasts queue
    let final_json = serde_json::to_string(&final_msg)
        .map_err(|e| anyhow::anyhow!("Failed to serialize final message: {}", e))?;

    producer
        .publish_event_json(&final_json, Some("broadcasts"))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to publish final progress: {}", e))?;

    tracing::info!(
        "✓ Avatar upload completed for user {}: {}",
        user_id,
        file_name
    );

    Ok(())
}
