mod delete_refresh_tokens_by_email;
mod seed;

use async_trait::async_trait;
use axum::http::StatusCode;

use crate::config::setting::Setting;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunbookMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunbookExecutionResult {
    pub name: &'static str,
    pub message: String,
}

impl RunbookExecutionResult {
    pub fn new(name: &'static str, message: impl Into<String>) -> Self {
        Self {
            name,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunbookError {
    pub status: StatusCode,
    pub message: String,
}

impl RunbookError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, message)
    }

    pub fn internal_error(err: impl std::fmt::Display) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal Server Error: {}", err),
        )
    }
}

impl std::fmt::Display for RunbookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}> {}", self.status.as_u16(), self.message)
    }
}

impl std::error::Error for RunbookError {}

#[async_trait]
trait Runbook: Send + Sync {
    fn metadata(&self) -> RunbookMetadata;
    async fn run(
        &self,
        setting: &Setting,
        args: &[String],
    ) -> Result<RunbookExecutionResult, RunbookError>;
}

fn all() -> Vec<Box<dyn Runbook>> {
    vec![
        Box::new(seed::Seed),
        Box::new(delete_refresh_tokens_by_email::DeleteRefreshTokensByEmail),
    ]
}

pub fn list() -> Vec<RunbookMetadata> {
    all()
        .into_iter()
        .map(|runbook| runbook.metadata())
        .collect()
}

pub async fn run(
    setting: &Setting,
    name: &str,
    args: &[String],
) -> Result<RunbookExecutionResult, RunbookError> {
    let runbook = all()
        .into_iter()
        .find(|runbook| runbook.metadata().name == name)
        .ok_or_else(|| RunbookError::not_found(format!("Unknown runbook: {name}")))?;

    runbook.run(setting, args).await
}

#[cfg(test)]
mod tests {
    use super::list;

    #[test]
    fn lists_available_runbooks_with_metadata() {
        let runbooks = list();

        assert!(runbooks.iter().any(|runbook| runbook.name == "seed"));
        assert!(
            runbooks
                .iter()
                .any(|runbook| runbook.name == "delete-refresh-tokens-by-email")
        );
        assert!(runbooks.iter().all(|runbook| {
            !runbook.name.is_empty() && !runbook.description.is_empty() && !runbook.usage.is_empty()
        }));
    }
}
