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

#[cfg(test)]
mod tests {
    use axum::{
        http::{HeaderMap, HeaderValue, StatusCode},
        response::IntoResponse,
    };
    use serde::Serialize;

    use super::ResponseDTO;

    #[derive(Debug, Serialize, PartialEq)]
    struct TestData {
        id: u32,
        name: &'static str,
    }

    #[test]
    fn creates_response_dto() {
        let response = ResponseDTO::new(
            StatusCode::OK,
            TestData {
                id: 1,
                name: "test",
            },
        );

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.data.id, 1);
    }

    #[test]
    fn includes_headers_when_present() {
        let mut headers = HeaderMap::new();
        headers.insert("x-test", HeaderValue::from_static("ok"));

        let response = ResponseDTO::with_headers(StatusCode::CREATED, "done", headers);
        let response = response.into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(response.headers()["x-test"], "ok");
    }
}
