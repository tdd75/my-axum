use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};

use crate::{
    config::setting::Setting,
    core::db::connection::get_db,
    user::entity::{refresh_token, user},
};

use super::{Runbook, RunbookError, RunbookExecutionResult, RunbookMetadata};

const RUNBOOK_NAME: &str = "delete-refresh-tokens-by-email";

pub struct DeleteRefreshTokensByEmail;

#[async_trait]
impl Runbook for DeleteRefreshTokensByEmail {
    fn metadata(&self) -> RunbookMetadata {
        RunbookMetadata {
            name: RUNBOOK_NAME,
            description: "Delete every refresh token owned by the user with the provided email",
            usage: "runbook run delete-refresh-tokens-by-email --email user@example.com",
        }
    }

    async fn run(
        &self,
        setting: &Setting,
        args: &[String],
    ) -> Result<RunbookExecutionResult, RunbookError> {
        let email = parse_email_arg(args)?;
        delete_refresh_tokens_by_email(setting, &email).await
    }
}

fn parse_email_arg(args: &[String]) -> Result<String, RunbookError> {
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        if arg == "--email" {
            let email = args
                .next()
                .ok_or_else(|| RunbookError::bad_request("Missing value for --email"))?;
            return Ok(email.to_string());
        }
    }

    Err(RunbookError::bad_request(
        "Missing required argument: --email <user@example.com>",
    ))
}

async fn delete_refresh_tokens_by_email(
    setting: &Setting,
    email: &str,
) -> Result<RunbookExecutionResult, RunbookError> {
    let db = get_db(&setting.database_url)
        .await
        .map_err(RunbookError::internal_error)?;
    let txn = db.begin().await.map_err(RunbookError::internal_error)?;

    let user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(&txn)
        .await
        .map_err(RunbookError::internal_error)?
        .ok_or_else(|| RunbookError::not_found(format!("User not found for email: {email}")))?;

    let delete_result = refresh_token::Entity::delete_many()
        .filter(refresh_token::Column::UserId.eq(user.id))
        .exec(&txn)
        .await
        .map_err(RunbookError::internal_error)?;

    txn.commit().await.map_err(RunbookError::internal_error)?;
    db.close().await.map_err(RunbookError::internal_error)?;

    Ok(RunbookExecutionResult::new(
        RUNBOOK_NAME,
        format!(
            "Deleted {} refresh token(s) for user '{}'",
            delete_result.rows_affected, email
        ),
    ))
}
