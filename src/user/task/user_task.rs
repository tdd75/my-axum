use chrono::Datelike;
use rust_i18n::t;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::{
    config::setting::{MessageType, Setting},
    core::{
        r#async::{TaskType, publish_task},
        context::Context,
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
    let txn = db.begin().await?;
    let txn = Arc::new(txn);
    let context = Context::builder(txn.clone()).build();

    let user = user_repository::find_by_id(&context, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to find user: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    drop(context);
    Arc::try_unwrap(txn)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap transaction for commit"))?
        .commit()
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
    locale: String,
) -> anyhow::Result<()> {
    tracing::info!(
        "Processing avatar upload for task_id: {}, user_id: {}, file_name: {}, locale: {}",
        task_id,
        user_id,
        file_name,
        locale
    );

    // Verify user exists
    let txn = db.begin().await?;
    let txn_arc = Arc::new(txn);
    let context = Context::builder(txn_arc.clone()).build();

    user_repository::find_by_id(&context, user_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to find user: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    drop(context);
    Arc::try_unwrap(txn_arc)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap transaction for commit"))?
        .commit()
        .await?;

    // Simulate upload progress with sleep
    let stages = vec![
        (
            10,
            t!(
                "avatar_upload.progress.validating_file",
                locale = locale.as_str()
            )
            .to_string(),
            100,
        ),
        (
            25,
            t!(
                "avatar_upload.progress.preparing_upload",
                locale = locale.as_str()
            )
            .to_string(),
            200,
        ),
        (
            40,
            t!(
                "avatar_upload.progress.processing_image",
                locale = locale.as_str()
            )
            .to_string(),
            400,
        ),
        (
            60,
            t!(
                "avatar_upload.progress.optimizing",
                locale = locale.as_str()
            )
            .to_string(),
            500,
        ),
        (
            80,
            t!(
                "avatar_upload.progress.finalizing",
                locale = locale.as_str()
            )
            .to_string(),
            300,
        ),
        (
            100,
            t!(
                "avatar_upload.progress.upload_complete",
                locale = locale.as_str()
            )
            .to_string(),
            200,
        ),
    ];

    for (progress, message, delay_ms) in stages {
        sleep(Duration::from_millis(delay_ms)).await;

        let progress_dto =
            AvatarUploadProgressDTO::new(task_id.clone(), user_id, progress, "processing")
                .with_message(&message);

        let broadcast_msg = BroadcastMessage {
            event_type: "avatar_upload_progress".to_string(),
            data: serde_json::to_value(&progress_dto)
                .map_err(|e| anyhow::anyhow!("Failed to serialize progress: {}", e))?,
        };

        // Cache task status in Redis (for late WebSocket connections)
        if let Err(e) = cache_task_status(redis_url, &task_id, &broadcast_msg).await {
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
            &message
        );
    }

    // Final success message
    let final_progress = AvatarUploadProgressDTO::new(task_id.clone(), user_id, 100, "completed")
        .with_message(
            t!(
                "avatar_upload.progress.uploaded_successfully",
                locale = locale.as_str(),
                file_name = file_name.as_str()
            )
            .as_ref(),
        );

    let final_msg = BroadcastMessage {
        event_type: "avatar_upload_complete".to_string(),
        data: serde_json::to_value(&final_progress)
            .map_err(|e| anyhow::anyhow!("Failed to serialize final progress: {}", e))?,
    };

    // Cache final task status in Redis (for late WebSocket connections)
    if let Err(e) = cache_task_status(redis_url, &task_id, &final_msg).await {
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
