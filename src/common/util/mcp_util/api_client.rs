use axum::http::{StatusCode, header};
use rmcp::{ErrorData as McpError, RoleServer, service::RequestContext};
use serde_json::{Value, json};

use crate::pkg::url::url_encode;

use super::error::mcp_error_code_from_status;
use http::request::Parts;

#[derive(Clone)]
pub struct HttpApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get(
        &self,
        ctx: &RequestContext<RoleServer>,
        path: &str,
        query: Option<&str>,
    ) -> Result<Value, McpError> {
        let parts = ctx.extensions.get::<Parts>().ok_or_else(|| {
            McpError::internal_error(
                "missing_http_request_parts",
                Some(json!({ "reason": "HTTP request parts were not available" })),
            )
        })?;
        let mut request = self.client.get(self.url(path, query));

        for name in [
            header::AUTHORIZATION,
            header::ACCEPT_LANGUAGE,
            header::COOKIE,
        ] {
            if let Some(value) = parts.headers.get(&name)
                && let Ok(value) = value.to_str()
            {
                request = request.header(name.as_str(), value);
            }
        }

        request = request.header(header::ACCEPT.as_str(), "application/json");

        let response = request.send().await.map_err(internal_mcp_error)?;
        let status = response.status();
        let body = response.text().await.map_err(internal_mcp_error)?;

        if status.is_success() {
            return serde_json::from_str(&body).map_err(internal_mcp_error);
        }

        let message = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|body| {
                body.get("message")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .unwrap_or_else(|| status.to_string());

        Err(http_error_to_mcp_error(status.as_u16(), message))
    }

    fn url(&self, path: &str, query: Option<&str>) -> String {
        let path = path
            .split('/')
            .map(url_encode)
            .collect::<Vec<String>>()
            .join("/");
        let mut url = format!("{}{}", self.base_url, path);

        if let Some(query) = query
            && !query.is_empty()
        {
            url.push('?');
            url.push_str(query);
        }

        url
    }
}

pub fn internal_mcp_error(error: impl std::fmt::Display) -> McpError {
    McpError::internal_error(error.to_string(), Some(Value::Null))
}

fn http_error_to_mcp_error(status: u16, message: String) -> McpError {
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    McpError::new(mcp_error_code_from_status(status), message, None)
}
