use axum::http::StatusCode;
use rmcp::{ErrorData, model::ErrorCode};

use crate::core::dto::error_dto::ErrorDTO;

pub fn mcp_error_code_from_status(status: StatusCode) -> ErrorCode {
    match status {
        StatusCode::BAD_REQUEST => ErrorCode::INVALID_PARAMS,
        StatusCode::NOT_FOUND => ErrorCode::RESOURCE_NOT_FOUND,
        StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => ErrorCode(-32000),
        StatusCode::INTERNAL_SERVER_ERROR => ErrorCode::INTERNAL_ERROR,
        _ => ErrorCode::INTERNAL_ERROR,
    }
}

impl From<ErrorDTO> for ErrorData {
    fn from(error: ErrorDTO) -> Self {
        ErrorData::new(
            mcp_error_code_from_status(error.status),
            error.message,
            None,
        )
    }
}
