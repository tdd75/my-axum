use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

use crate::core::dto::util::{ToJson, serialize_status_code};

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
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal Server Error: {}", e),
        )
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| {
            json!({
                "error": "Failed to serialize error"
            })
        })
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| r#"{"error":"Failed to serialize error"}"#.to_string())
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

impl ToJson for ErrorDTO {}
