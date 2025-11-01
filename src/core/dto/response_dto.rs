use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::core::dto::util::{ToJson, serialize_status_code};

#[derive(Debug, Serialize)]
pub struct ResponseDTO<T: Serialize> {
    #[serde(serialize_with = "serialize_status_code")]
    pub status: StatusCode,
    pub data: T,
    #[serde(skip_serializing)]
    pub headers: Option<HeaderMap>,
}

impl<T: Serialize> ResponseDTO<T> {
    pub fn new(status: StatusCode, data: T) -> Self {
        Self {
            status,
            data,
            headers: None,
        }
    }

    pub fn with_headers(status: StatusCode, data: T, headers: HeaderMap) -> Self {
        Self {
            status,
            data,
            headers: Some(headers),
        }
    }
}

impl<T: Serialize> IntoResponse for ResponseDTO<T> {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = Json(self.data);

        if let Some(headers) = self.headers {
            (status, headers, body).into_response()
        } else {
            (status, body).into_response()
        }
    }
}

impl<T: Serialize> ToJson for ResponseDTO<T> {}
