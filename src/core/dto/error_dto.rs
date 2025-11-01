use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;
use std::backtrace::Backtrace;
use utoipa::ToSchema;

use rust_i18n::t;

use crate::core::dto::util::{ToJson, serialize_status_code};
use crate::core::runbook::RunbookError;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDTO {
    #[schema(value_type = u64)]
    #[serde(serialize_with = "serialize_status_code")]
    pub status: StatusCode,
    pub message: String,
}

impl ErrorDTO {
    pub fn new(status: StatusCode, message: String) -> Self {
        Self { status, message }
    }

    pub fn map_internal_error(e: impl std::fmt::Display) -> Self {
        let backtrace = Backtrace::force_capture();
        tracing::error!(error = %e, backtrace = %backtrace, "Internal error mapped to response");

        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            t!("common.internal_server_error", error = e, locale = "en").to_string(),
        )
    }
}

impl IntoResponse for ErrorDTO {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = Json(json!({
            "message": self.message,
        }));
        (status, body).into_response()
    }
}

impl std::error::Error for ErrorDTO {}

impl std::fmt::Display for ErrorDTO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}> {}", self.status.as_u16(), self.message)
    }
}

impl From<sea_orm::DbErr> for ErrorDTO {
    fn from(err: sea_orm::DbErr) -> Self {
        Self::map_internal_error(err)
    }
}

impl From<RunbookError> for ErrorDTO {
    fn from(err: RunbookError) -> Self {
        Self::new(err.status, err.message)
    }
}

impl ToJson for ErrorDTO {}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};

    use super::ErrorDTO;
    use crate::core::runbook::RunbookError;

    #[test]
    fn creates_and_formats_error() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Test error".to_string());
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Test error");
        assert_eq!(format!("{error}"), "<400> Test error");
    }

    #[test]
    fn maps_internal_error() {
        let error = ErrorDTO::map_internal_error("db failed");
        assert_eq!(error.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(error.message.contains("db failed"));
    }

    #[test]
    fn converts_into_response() {
        let response = ErrorDTO::new(StatusCode::NOT_FOUND, "Missing".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn converts_from_runbook_error() {
        let error = ErrorDTO::from(RunbookError::bad_request("invalid args"));
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "invalid args");
    }
}
