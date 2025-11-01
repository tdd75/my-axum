use std::collections::HashMap;

use crate::core::db::entity::password_reset_token;
use crate::{
    config::setting::{MessageType, Setting},
    core::{
        context::Context,
        db::entity::user,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
        task::{TaskPriority, TaskType, publish_task_with_priority},
        template::engine::render_email_template,
    },
    user::{
        dto::auth_dto::ForgotPasswordDTO,
        repository::{password_reset_repository, user_repository},
        service::user_service,
    },
};
use axum::http::StatusCode;
use chrono::Datelike;
use chrono::Duration;
use rand::Rng;
use sea_orm::Set;

pub async fn execute(
    context: &Context<'_>,
    dto: ForgotPasswordDTO,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    // Validate email format
    user_service::validate_email_format(&dto.email)?;

    // Find user by email
    let user = user_repository::find_by_email(context, &dto.email)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    // If user doesn't exist, still return success to prevent email enumeration
    if user.is_none() {
        tracing::warn!(
            "Password reset requested for non-existent email: {}",
            dto.email
        );
        return Ok(ResponseDTO::new(StatusCode::NO_CONTENT, ()));
    }

    let user = user.unwrap();

    // Delete any existing reset tokens for this user
    password_reset_repository::delete_by_user_id(context, user.id)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    // Generate a secure 6-digit OTP
    let otp: String = {
        let mut rng = rand::rng();
        format!("{:06}", rng.random_range(0..1000000))
    };

    // Save OTP to database
    let expires_at = chrono::Utc::now() + Duration::minutes(15);
    let reset_token = password_reset_token::ActiveModel {
        user_id: Set(user.id),
        token: Set(otp.clone()),
        retry_count: Set(0),
        expires_at: Set(expires_at.naive_utc()),
        ..Default::default()
    };
    password_reset_repository::create(context, reset_token)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    // Send password reset email
    send_forgot_password_email(context, &user, &otp).await?;

    tracing::info!("Password reset process completed for email: {}", dto.email);
    Ok(ResponseDTO::new(StatusCode::NO_CONTENT, ()))
}

async fn send_forgot_password_email(
    context: &Context<'_>,
    user: &user::Model,
    otp: &str,
) -> Result<(), ErrorDTO> {
    // Get app settings
    let setting = Setting::new();

    // Prepare template variables
    let mut variables = HashMap::new();
    variables.insert("app_name".to_string(), "My Axum App".to_string());
    variables.insert("app_url".to_string(), setting.app_url.clone());
    variables.insert("email".to_string(), user.email.clone());
    variables.insert(
        "first_name".to_string(),
        if let Some(ref name) = user.first_name {
            format!(" {}", name)
        } else {
            String::new()
        },
    );
    variables.insert("otp".to_string(), otp.to_string());
    variables.insert("expiry_minutes".to_string(), "15".to_string());
    variables.insert("max_attempts".to_string(), "3".to_string());
    variables.insert("year".to_string(), chrono::Utc::now().year().to_string());

    // Render HTML template
    let html_body = render_email_template("email/password_reset.html", variables).map_err(|e| {
        tracing::error!("Failed to render password reset email template: {}", e);
        ErrorDTO::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to prepare email".to_string(),
        )
    })?;

    // Publish password reset email task with HIGH priority
    match &context.producer {
        Some(producer) => {
            if let Err(e) = publish_task_with_priority(
                producer.as_ref().as_ref(),
                TaskType::SendEmail {
                    to: user.email.clone(),
                    subject: "Password Reset Request - My Axum App".to_string(),
                    text_body: None,
                    html_body: Some(html_body),
                },
                TaskPriority::High, // HIGH PRIORITY for password reset emails
                Some(MessageType::Emails.as_ref()),
            )
            .await
            {
                tracing::error!("Failed to publish password reset email task: {}", e);
                return Err(ErrorDTO::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to send password reset email".to_string(),
                ));
            }

            tracing::info!(
                "✓ Password reset email task published with HIGH priority for: {}",
                user.email
            );
        }
        None => {
            tracing::error!("Message producer not available. Cannot send password reset email.");
            return Err(ErrorDTO::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Email service unavailable".to_string(),
            ));
        }
    }
    Ok(())
}
