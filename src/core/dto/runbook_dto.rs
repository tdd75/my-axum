use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::core::runbook::{RunbookExecutionResult, RunbookMetadata};

#[derive(Debug, Serialize, ToSchema)]
pub struct RunbookInfoDTO {
    pub name: String,
    pub description: String,
    pub usage: String,
}

impl From<RunbookMetadata> for RunbookInfoDTO {
    fn from(metadata: RunbookMetadata) -> Self {
        Self {
            name: metadata.name.to_string(),
            description: metadata.description.to_string(),
            usage: metadata.usage.to_string(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RunbookListDTO {
    pub runbooks: Vec<RunbookInfoDTO>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RunRunbookRequestDTO {
    pub name: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RunRunbookResponseDTO {
    pub name: String,
    pub message: String,
}

impl From<RunbookExecutionResult> for RunRunbookResponseDTO {
    fn from(result: RunbookExecutionResult) -> Self {
        Self {
            name: result.name.to_string(),
            message: result.message,
        }
    }
}
