use axum::http::{HeaderMap, StatusCode};
use reqwest::{Client, Method};
use rust_i18n::t;
use serde_json::{Value, json};
use utoipa::OpenApi;

use crate::{
    assistant::dto::assistant_dto::{
        AssistantChatMessageDTO, AssistantChatMessageRoleDTO, AssistantPlanDTO,
        AssistantPlannedApiCallDTO,
    },
    config::app::AppState,
    core::{api::openapi::ApiDoc, dto::error_dto::ErrorDTO},
};

const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
const MAX_USER_MESSAGE_LEN: usize = 2_000;
const MAX_CONTEXT_MESSAGES: usize = 12;

pub fn validate_message(message: &str, locale: &str) -> Result<(), ErrorDTO> {
    if message.trim().is_empty() {
        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            t!("assistant.validation.message_required", locale = locale).to_string(),
        ));
    }

    if message.chars().count() > MAX_USER_MESSAGE_LEN {
        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            t!(
                "assistant.validation.message_too_long",
                max = MAX_USER_MESSAGE_LEN,
                locale = locale
            )
            .to_string(),
        ));
    }

    Ok(())
}

pub fn validate_context_messages(
    messages: &[AssistantChatMessageDTO],
    locale: &str,
) -> Result<(), ErrorDTO> {
    if messages.len() > MAX_CONTEXT_MESSAGES {
        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            t!(
                "assistant.validation.context_too_many_messages",
                max = MAX_CONTEXT_MESSAGES,
                locale = locale
            )
            .to_string(),
        ));
    }

    for message in messages {
        if message.content.chars().count() > MAX_USER_MESSAGE_LEN {
            return Err(ErrorDTO::new(
                StatusCode::BAD_REQUEST,
                t!(
                    "assistant.validation.context_message_too_long",
                    max = MAX_USER_MESSAGE_LEN,
                    locale = locale
                )
                .to_string(),
            ));
        }
    }

    Ok(())
}

pub fn openapi_json() -> Result<Value, ErrorDTO> {
    let openapi = ApiDoc::openapi();
    serde_json::to_value(&openapi).map_err(ErrorDTO::map_internal_error)
}

pub async fn plan_api_call(
    client: &Client,
    api_key: &str,
    model: &str,
    message: &str,
    messages: &[AssistantChatMessageDTO],
    openapi_json: &Value,
    locale: &str,
) -> Result<AssistantPlanDTO, ErrorDTO> {
    let instructions = format!(
        r#"You are an API assistant for this Axum app.
Use only this OpenAPI document to decide whether an API call is needed:
{openapi_json}

Return JSON only with this exact shape:
{{"answer": string|null, "api_call": {{"method": "GET|POST|PATCH|DELETE", "path": "/api/v1/...", "query": {{}}, "body": {{}} }}|null}}

Rules:
- Pick at most one HTTP API call.
- Use concrete paths. Replace path parameters like {{id}} only when the user provided a value.
- If required information is missing from current_message, inspect conversation_history before asking for it.
- If the latest assistant message asked for missing fields and current_message supplies those fields or values, treat current_message as a follow-up answer and produce the API call instead of asking again.
- Accept natural field answers such as `first_name=Admin`, `last_name=User`, `first_name: Admin`, `last_name: User`, or comma-separated field assignments as valid values.
- If current_message provides a complete display name for profile updates, map it to the documented profile fields instead of asking for field names.
- Set api_call to null only when the current request cannot be completed with the OpenAPI document plus conversation_history.
- Never invent endpoints, fields, or enum values.
- Do not call destructive or credential-changing APIs unless the user clearly requested it and provided required inputs.
- Use conversation_history to resolve references like "that", "again", "the old value", "continue", or omitted fields.
- The current_message is the user's latest request and should take priority over earlier conversation history.
- The caller is already authenticated; prefer /api/v1/user/profile/ for questions about the current user."#
    );

    let output = create_openai_response(
        client,
        api_key,
        model,
        &instructions,
        &build_planning_input(message, messages),
        locale,
    )
    .await?;
    parse_json_from_model_output(&output, locale)
}

fn build_planning_input(message: &str, messages: &[AssistantChatMessageDTO]) -> String {
    let context = messages
        .iter()
        .rev()
        .take(MAX_CONTEXT_MESSAGES)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|message| {
            let role = match message.role {
                AssistantChatMessageRoleDTO::User => "user",
                AssistantChatMessageRoleDTO::Assistant => "assistant",
            };

            json!({
                "role": role,
                "content": message.content,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "conversation_history": context,
        "current_message": message,
    })
    .to_string()
}

pub fn format_no_api_answer(answer: Option<&str>, locale: &str) -> String {
    let answer = answer.unwrap_or("").trim();

    if !answer.is_empty() && !contains_technical_detail(answer) {
        return answer.to_string();
    }

    t!("assistant.response.need_more_information", locale = locale).to_string()
}

pub fn format_api_result_for_user(api_result: &AssistantApiResult, locale: &str) -> String {
    if !(200..300).contains(&api_result.status) {
        return format_failure_message(&api_result.body, locale);
    }

    t!("assistant.response.done", locale = locale).to_string()
}

pub async fn refine_answer_with_openai(
    client: &Client,
    api_key: &str,
    model: &str,
    user_message: &str,
    messages: &[AssistantChatMessageDTO],
    draft_answer: &str,
    locale: &str,
) -> Result<String, ErrorDTO> {
    let instructions = r#"Rewrite the assistant draft reply for the user.
Rules:
- Keep the exact meaning of the draft.
- Use the same language as the user's latest message and conversation context.
- Be concise and natural.
- Do not add technical details, API paths, HTTP methods, status codes, or JSON fields.
- Return plain text only."#;

    let input = json!({
        "conversation_history": messages,
        "current_message": user_message,
        "draft_answer": draft_answer,
    })
    .to_string();

    let refined =
        create_openai_response(client, api_key, model, instructions, &input, locale).await?;
    let refined = refined.trim();

    if refined.is_empty() {
        return Ok(draft_answer.to_string());
    }

    Ok(refined.to_string())
}

pub async fn refine_api_result_answer_with_openai(
    client: &Client,
    api_key: &str,
    model: &str,
    user_message: &str,
    messages: &[AssistantChatMessageDTO],
    refinement: AssistantApiResultRefinement<'_>,
    locale: &str,
) -> Result<String, ErrorDTO> {
    let instructions = r#"Write the assistant reply for the user after an internal API call.
Rules:
- Use the same language as the user's latest message and conversation context.
- If api_succeeded is true and api_method is GET, answer from api_result. Do not answer with the draft unless api_result is unusable.
- If api_succeeded is true and api_method is not GET, use api_result when it contains useful user-facing data; otherwise keep the meaning of the draft success message.
- If api_result contains an empty list, say that no matching result was found.
- Be concise and natural.
- Do not mention technical details, API paths, HTTP methods, status codes, JSON fields, or internal implementation.
- If the API call failed, keep the meaning of the draft failure message.
- Return plain text only."#;

    let input = build_api_result_refinement_input(
        user_message,
        messages,
        refinement.api_call,
        refinement.api_result,
        refinement.draft_answer,
    );

    let refined =
        create_openai_response(client, api_key, model, instructions, &input, locale).await?;
    Ok(format_refined_api_answer(&refined, refinement.draft_answer))
}

pub struct AssistantApiResultRefinement<'a> {
    pub api_call: &'a AssistantPlannedApiCallDTO,
    pub api_result: &'a AssistantApiResult,
    pub draft_answer: &'a str,
}

fn build_api_result_refinement_input(
    user_message: &str,
    messages: &[AssistantChatMessageDTO],
    api_call: &AssistantPlannedApiCallDTO,
    api_result: &AssistantApiResult,
    draft_answer: &str,
) -> String {
    json!({
        "conversation_history": messages,
        "current_message": user_message,
        "api_method": api_call.method.to_uppercase(),
        "api_result": api_result.body,
        "api_succeeded": (200..300).contains(&api_result.status),
        "draft_answer": draft_answer,
    })
    .to_string()
}

fn format_refined_api_answer(refined: &str, draft_answer: &str) -> String {
    let refined = refined.trim();

    if refined.is_empty() || contains_technical_detail(refined) {
        return draft_answer.to_string();
    }

    refined.to_string()
}

pub async fn create_openai_response(
    client: &Client,
    api_key: &str,
    model: &str,
    instructions: &str,
    input: &str,
    locale: &str,
) -> Result<String, ErrorDTO> {
    create_openai_response_to_url(
        client,
        OPENAI_RESPONSES_URL,
        api_key,
        model,
        instructions,
        input,
        locale,
    )
    .await
}

async fn create_openai_response_to_url(
    client: &Client,
    url: &str,
    api_key: &str,
    model: &str,
    instructions: &str,
    input: &str,
    locale: &str,
) -> Result<String, ErrorDTO> {
    let response = client
        .post(url)
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "instructions": instructions,
            "input": input,
            "store": false,
        }))
        .send()
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(ErrorDTO::map_internal_error)?;
    if !status.is_success() {
        let message = body
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or(t!("assistant.error.openai_request_failed", locale = locale).as_ref())
            .to_string();
        return Err(ErrorDTO::new(StatusCode::BAD_GATEWAY, message));
    }

    extract_response_text(&body).ok_or_else(|| {
        ErrorDTO::new(
            StatusCode::BAD_GATEWAY,
            t!(
                "assistant.error.openai_response_missing_text",
                locale = locale
            )
            .to_string(),
        )
    })
}

fn extract_response_text(body: &Value) -> Option<String> {
    if let Some(text) = body.get("output_text").and_then(Value::as_str) {
        return Some(text.to_string());
    }

    let text = body
        .get("output")?
        .as_array()?
        .iter()
        .filter_map(|item| item.get("content").and_then(Value::as_array))
        .flatten()
        .filter_map(|content| content.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join("");

    (!text.is_empty()).then_some(text)
}

fn format_failure_message(body: &Value, locale: &str) -> String {
    match body.get("message").and_then(Value::as_str) {
        Some(message) => t!(
            "assistant.response.failure_with_message",
            message = message,
            locale = locale
        )
        .to_string(),
        None => t!("assistant.response.failure", locale = locale).to_string(),
    }
}

fn contains_technical_detail(answer: &str) -> bool {
    let normalized = answer.to_lowercase();
    [
        "/api",
        "endpoint",
        "swagger",
        "openapi",
        "http",
        "json",
        "patch",
        "post",
        "get ",
        "delete",
        "status code",
        "request body",
        "response body",
        "first_name",
        "last_name",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn parse_json_from_model_output<T: serde::de::DeserializeOwned>(
    output: &str,
    locale: &str,
) -> Result<T, ErrorDTO> {
    let trimmed = output.trim();
    match serde_json::from_str(trimmed) {
        Ok(parsed) => Ok(parsed),
        Err(_) => {
            let start = trimmed.find('{').ok_or_else(|| {
                ErrorDTO::new(
                    StatusCode::BAD_GATEWAY,
                    t!("assistant.error.model_output_not_json", locale = locale).to_string(),
                )
            })?;
            let end = trimmed.rfind('}').ok_or_else(|| {
                ErrorDTO::new(
                    StatusCode::BAD_GATEWAY,
                    t!("assistant.error.model_output_not_json", locale = locale).to_string(),
                )
            })?;
            serde_json::from_str(&trimmed[start..=end]).map_err(ErrorDTO::map_internal_error)
        }
    }
}

pub fn validate_api_call(
    openapi_json: &Value,
    api_call: &AssistantPlannedApiCallDTO,
    locale: &str,
) -> Result<(), ErrorDTO> {
    let method = api_call.method.to_lowercase();
    if !matches!(method.as_str(), "get" | "post" | "patch" | "delete") {
        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            t!("assistant.error.unsupported_api_method", locale = locale).to_string(),
        ));
    }

    let paths = openapi_json
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| ErrorDTO::map_internal_error("OpenAPI paths are missing"))?;

    let allowed = paths.iter().any(|(template, operations)| {
        path_matches(template, &api_call.path)
            && operations
                .get(&method)
                .map(|operation| !operation.is_null())
                .unwrap_or(false)
    });

    if !allowed || api_call.path.contains("/api/v1/assistant/") {
        return Err(ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            t!("assistant.error.api_call_not_in_openapi", locale = locale).to_string(),
        ));
    }

    Ok(())
}

fn path_matches(template: &str, path: &str) -> bool {
    let template_parts: Vec<&str> = template.trim_matches('/').split('/').collect();
    let path_parts: Vec<&str> = path.trim_matches('/').split('/').collect();

    template_parts.len() == path_parts.len()
        && template_parts
            .iter()
            .zip(path_parts.iter())
            .all(|(left, right)| (left.starts_with('{') && left.ends_with('}')) || left == right)
}

pub struct AssistantApiResult {
    pub status: u16,
    pub body: Value,
}

pub async fn execute_api_call(
    app_state: &AppState,
    client: &Client,
    headers: &HeaderMap,
    api_call: &AssistantPlannedApiCallDTO,
    locale: &str,
) -> Result<AssistantApiResult, ErrorDTO> {
    let method = Method::from_bytes(api_call.method.to_uppercase().as_bytes()).map_err(|_| {
        ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            t!("assistant.error.unsupported_api_method", locale = locale).to_string(),
        )
    })?;
    let url = build_internal_url(app_state, api_call)?;
    let mut request = client.request(method, url);

    for name in ["authorization", "cookie", "accept-language"] {
        if let Some(value) = headers.get(name)
            && let Ok(value) = value.to_str()
        {
            request = request.header(name, value);
        }
    }

    if !api_call.body.is_null() && api_call.body != json!({}) {
        request = request.json(&api_call.body);
    }

    let response = request.send().await.map_err(ErrorDTO::map_internal_error)?;
    let status = response.status().as_u16();
    let text = response
        .text()
        .await
        .map_err(ErrorDTO::map_internal_error)?;
    let body = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "text": text }));

    Ok(AssistantApiResult { status, body })
}

fn build_internal_url(
    app_state: &AppState,
    api_call: &AssistantPlannedApiCallDTO,
) -> Result<String, ErrorDTO> {
    let host = if app_state.setting.app_host.contains(':')
        && !app_state.setting.app_host.starts_with('[')
    {
        format!("[{}]", app_state.setting.app_host)
    } else {
        app_state.setting.app_host.clone()
    };
    let base = format!("http://{}:{}", host, app_state.setting.app_port);
    let mut url = reqwest::Url::parse(&base)
        .and_then(|base| base.join(api_call.path.trim_start_matches('/')))
        .map_err(ErrorDTO::map_internal_error)?;

    if let Some(query) = api_call.query.as_object() {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in query {
            if value.is_null() {
                continue;
            }
            if let Some(value) = value.as_str() {
                pairs.append_pair(key, value);
            } else {
                pairs.append_pair(key, &value.to_string());
            }
        }
    }

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use axum::{Json, Router, http::StatusCode, routing::post};
    use reqwest::Client;
    use serde_json::json;

    use super::{
        AssistantApiResult, AssistantPlannedApiCallDTO, build_api_result_refinement_input,
        build_internal_url, build_planning_input, create_openai_response_to_url,
        extract_response_text, format_api_result_for_user, format_no_api_answer,
        format_refined_api_answer, parse_json_from_model_output, path_matches, validate_api_call,
        validate_context_messages, validate_message,
    };
    use crate::assistant::dto::assistant_dto::{
        AssistantChatMessageDTO, AssistantChatMessageRoleDTO, AssistantPlanDTO,
    };
    use crate::config::{app::AppState, setting::Setting};
    use crate::core::db::connection::get_db;

    async fn spawn_openai_test_server(
        response_status: StatusCode,
        response_body: serde_json::Value,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let app = Router::new().route(
            "/v1/responses",
            post(move |Json(_payload): Json<serde_json::Value>| {
                let response_body = response_body.clone();
                async move { (response_status, Json(response_body)) }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{addr}/v1/responses"), handle)
    }

    fn api_call(method: &str) -> AssistantPlannedApiCallDTO {
        AssistantPlannedApiCallDTO {
            method: method.to_string(),
            path: "/api/v1/user/".to_string(),
            query: json!({}),
            body: json!({}),
        }
    }

    #[test]
    fn validates_assistant_message_input() {
        let empty = validate_message("   ", "en").unwrap_err();
        assert_eq!(empty.status, StatusCode::BAD_REQUEST);

        let too_long = validate_message(&"a".repeat(2_001), "en").unwrap_err();
        assert_eq!(too_long.status, StatusCode::BAD_REQUEST);

        assert!(validate_message("show my profile", "en").is_ok());
    }

    #[test]
    fn validates_context_message_limits() {
        let messages = (0..13)
            .map(|index| AssistantChatMessageDTO {
                role: AssistantChatMessageRoleDTO::User,
                content: format!("message {index}"),
            })
            .collect::<Vec<_>>();

        let error = validate_context_messages(&messages, "en").unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);

        let messages = vec![AssistantChatMessageDTO {
            role: AssistantChatMessageRoleDTO::User,
            content: "a".repeat(2_001),
        }];
        let error = validate_context_messages(&messages, "en").unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn builds_planning_input_with_conversation_history() {
        let messages = vec![
            AssistantChatMessageDTO {
                role: AssistantChatMessageRoleDTO::User,
                content: "update my name to Admin 1".to_string(),
            },
            AssistantChatMessageDTO {
                role: AssistantChatMessageRoleDTO::Assistant,
                content: "Updated your name to Admin 1.".to_string(),
            },
        ];

        let input = build_planning_input("change it again to Admin User", &messages);
        let input: serde_json::Value = serde_json::from_str(&input).unwrap();

        assert_eq!(input["current_message"], "change it again to Admin User");
        assert_eq!(input["conversation_history"][0]["role"], "user");
        assert_eq!(
            input["conversation_history"][1]["content"],
            "Updated your name to Admin 1."
        );
    }

    #[test]
    fn extracts_text_from_openai_response_shapes() {
        assert_eq!(
            extract_response_text(&json!({ "output_text": "hello" })).as_deref(),
            Some("hello")
        );

        let nested = json!({
            "output": [
                {
                    "content": [
                        { "type": "output_text", "text": "hello " },
                        { "type": "output_text", "text": "world" }
                    ]
                }
            ]
        });
        assert_eq!(
            extract_response_text(&nested).as_deref(),
            Some("hello world")
        );
    }

    #[tokio::test]
    async fn creates_openai_response_from_output_text() {
        let (url, handle) =
            spawn_openai_test_server(StatusCode::OK, json!({ "output_text": "hello" })).await;
        let client = Client::new();

        let response = create_openai_response_to_url(
            &client,
            &url,
            "test-key",
            "test-model",
            "inst",
            "input",
            "en",
        )
        .await
        .unwrap();

        assert_eq!(response, "hello");
        handle.abort();
    }

    #[tokio::test]
    async fn create_openai_response_surfaces_api_error_message() {
        let (url, handle) = spawn_openai_test_server(
            StatusCode::BAD_REQUEST,
            json!({ "error": { "message": "bad request" } }),
        )
        .await;
        let client = Client::new();

        let error = create_openai_response_to_url(
            &client,
            &url,
            "test-key",
            "test-model",
            "inst",
            "input",
            "en",
        )
        .await
        .unwrap_err();

        assert_eq!(error.status, StatusCode::BAD_GATEWAY);
        assert_eq!(error.message, "bad request");
        handle.abort();
    }

    #[tokio::test]
    async fn create_openai_response_requires_text_output() {
        let (url, handle) =
            spawn_openai_test_server(StatusCode::OK, json!({ "id": "resp_1" })).await;
        let client = Client::new();

        let error = create_openai_response_to_url(
            &client,
            &url,
            "test-key",
            "test-model",
            "inst",
            "input",
            "en",
        )
        .await
        .unwrap_err();

        assert_eq!(error.status, StatusCode::BAD_GATEWAY);
        assert_eq!(
            error.message,
            "I couldn't get a clear reply this time. Please try again."
        );
        handle.abort();
    }

    #[test]
    fn parses_json_from_noisy_model_output() {
        let parsed: AssistantPlanDTO = parse_json_from_model_output(
            r#"```json
            {"answer":"Need an id","api_call":null}
            ```"#,
            "en",
        )
        .unwrap();

        assert_eq!(parsed.answer.as_deref(), Some("Need an id"));
        assert!(parsed.api_call.is_none());
    }

    #[test]
    fn matches_openapi_path_templates() {
        assert!(path_matches("/api/v1/user/{id}/", "/api/v1/user/42/"));
        assert!(!path_matches(
            "/api/v1/user/{id}/",
            "/api/v1/user/42/avatar/"
        ));
    }

    #[test]
    fn validates_api_calls_against_openapi_paths() {
        let openapi = json!({
            "paths": {
                "/api/v1/user/profile/": { "get": {} },
                "/api/v1/user/{id}/": { "patch": {} },
                "/api/v1/assistant/chat/": { "post": {} }
            }
        });

        let valid = AssistantPlannedApiCallDTO {
            method: "GET".to_string(),
            path: "/api/v1/user/profile/".to_string(),
            query: json!({}),
            body: json!({}),
        };
        assert!(validate_api_call(&openapi, &valid, "en").is_ok());

        let path_param = AssistantPlannedApiCallDTO {
            method: "PATCH".to_string(),
            path: "/api/v1/user/12/".to_string(),
            query: json!({}),
            body: json!({ "first_name": "Jane" }),
        };
        assert!(validate_api_call(&openapi, &path_param, "en").is_ok());

        let assistant_recursion = AssistantPlannedApiCallDTO {
            method: "POST".to_string(),
            path: "/api/v1/assistant/chat/".to_string(),
            query: json!({}),
            body: json!({ "message": "loop" }),
        };
        assert!(validate_api_call(&openapi, &assistant_recursion, "en").is_err());

        let unknown = AssistantPlannedApiCallDTO {
            method: "DELETE".to_string(),
            path: "/api/v1/missing/".to_string(),
            query: json!({}),
            body: json!({}),
        };
        assert!(validate_api_call(&openapi, &unknown, "en").is_err());
    }

    #[test]
    fn validates_api_call_rejects_unsupported_method() {
        let openapi = json!({
            "paths": {
                "/api/v1/user/profile/": { "get": {} }
            }
        });

        let invalid_method = AssistantPlannedApiCallDTO {
            method: "PUT".to_string(),
            path: "/api/v1/user/profile/".to_string(),
            query: json!({}),
            body: json!({}),
        };

        let error = validate_api_call(&openapi, &invalid_method, "en").unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn parse_json_returns_bad_gateway_when_no_json_payload() {
        let error =
            parse_json_from_model_output::<AssistantPlanDTO>("not-json-at-all", "en").unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_GATEWAY);
        assert_eq!(
            error.message,
            "I couldn't process this request in the expected way."
        );
    }

    #[test]
    fn formats_successful_api_result_with_generic_draft() {
        let api_result = AssistantApiResult {
            status: 200,
            body: json!({
                "id": 1,
                "email": "admin@example.com",
                "first_name": "Admin",
                "last_name": "1",
                "phone": "+1987654321"
            }),
        };

        let response = format_api_result_for_user(&api_result, "en");

        assert_eq!(response, "Done.");
        for forbidden in ["API", "PATCH", "/api", "200", "first_name", "last_name"] {
            assert!(
                !response.contains(forbidden),
                "assistant response should not expose {forbidden}"
            );
        }
    }

    #[test]
    fn builds_api_result_refinement_input_with_response_body() {
        let messages = vec![
            AssistantChatMessageDTO {
                role: AssistantChatMessageRoleDTO::User,
                content: "What is my name?".to_string(),
            },
            AssistantChatMessageDTO {
                role: AssistantChatMessageRoleDTO::Assistant,
                content: "I can check that.".to_string(),
            },
        ];
        let api_result = AssistantApiResult {
            status: 200,
            body: json!({
                "email": "admin@example.com",
                "first_name": "Admin",
                "last_name": "User"
            }),
        };

        let input = build_api_result_refinement_input(
            "What is my name?",
            &messages,
            &api_call("GET"),
            &api_result,
            "Done.",
        );
        let input: serde_json::Value = serde_json::from_str(&input).unwrap();

        assert_eq!(input["current_message"], "What is my name?");
        assert_eq!(input["conversation_history"][0]["role"], "user");
        assert_eq!(input["api_method"], "GET");
        assert_eq!(input["api_result"]["first_name"], "Admin");
        assert_eq!(input["api_result"]["last_name"], "User");
        assert_eq!(input["api_succeeded"], true);
        assert_eq!(input["draft_answer"], "Done.");
    }

    #[test]
    fn builds_api_result_refinement_input_for_failed_call() {
        let api_result = AssistantApiResult {
            status: 400,
            body: json!({ "message": "Invalid name" }),
        };

        let input = build_api_result_refinement_input(
            "Update my name",
            &[],
            &api_call("PATCH"),
            &api_result,
            "I couldn't complete that: Invalid name",
        );
        let input: serde_json::Value = serde_json::from_str(&input).unwrap();

        assert_eq!(input["api_method"], "PATCH");
        assert_eq!(input["api_succeeded"], false);
        assert_eq!(input["api_result"]["message"], "Invalid name");
        assert_eq!(
            input["draft_answer"],
            "I couldn't complete that: Invalid name"
        );
    }

    #[test]
    fn refined_api_answer_falls_back_when_model_leaks_technical_details() {
        let response =
            format_refined_api_answer("Call PATCH /api/v1/user/profile/ with first_name.", "Done.");

        assert_eq!(response, "Done.");
    }

    #[test]
    fn refined_api_answer_keeps_natural_text() {
        let response = format_refined_api_answer("  Your name is Admin User.  ", "Done.");

        assert_eq!(response, "Your name is Admin User.");
    }

    #[test]
    fn replaces_no_api_answers_that_leak_technical_details() {
        let response = format_no_api_answer(
            Some("Please call PATCH /api/v1/user/profile/ with first_name."),
            "en",
        );

        assert_eq!(response, "I need a bit more information to help with that.");
        assert!(!response.contains("/api"));
        assert!(!response.contains("PATCH"));
    }

    #[test]
    fn keeps_non_technical_no_api_answer() {
        let response = format_no_api_answer(Some("I can help with that."), "en");
        assert_eq!(response, "I can help with that.");
    }

    #[test]
    fn format_api_result_handles_english_fallbacks() {
        let success_without_name = AssistantApiResult {
            status: 200,
            body: json!({ "email": "user@example.com" }),
        };
        let failure_without_message = AssistantApiResult {
            status: 500,
            body: json!({}),
        };

        assert_eq!(
            format_api_result_for_user(&success_without_name, "en"),
            "Done."
        );
        assert_eq!(
            format_api_result_for_user(&failure_without_message, "en"),
            "I couldn't complete that. Please try again."
        );
    }

    #[test]
    fn no_api_fallback_uses_conversation_language() {
        let response = format_no_api_answer(Some("Please provide first_name and last_name."), "en");

        assert_eq!(response, "I need a bit more information to help with that.");
    }

    #[test]
    fn formats_profile_lookup_as_generic_success_before_refinement() {
        let api_result = AssistantApiResult {
            status: 200,
            body: json!({
                "email": "admin@example.com",
                "first_name": "Admin",
                "last_name": "User"
            }),
        };

        let response = format_api_result_for_user(&api_result, "en");

        assert_eq!(response, "Done.");
        assert!(!response.contains("/api"));
        assert!(!response.contains("GET"));
    }

    #[test]
    fn formats_failure_without_http_status_details() {
        let api_result = AssistantApiResult {
            status: 400,
            body: json!({ "message": "Invalid name" }),
        };

        let response = format_api_result_for_user(&api_result, "en");

        assert_eq!(response, "I couldn't complete that: Invalid name");
        assert!(!response.contains("400"));
    }

    #[test]
    fn api_result_is_available_to_use_cases() {
        let result = AssistantApiResult {
            status: 200,
            body: json!({ "ok": true }),
        };

        assert_eq!(result.status, 200);
        assert_eq!(result.body["ok"], true);
    }

    #[tokio::test]
    async fn build_internal_url_includes_query_params() {
        let db = get_db("sqlite::memory:").await.unwrap();
        let mut setting = Setting::new();
        setting.app_host = "localhost".to_string();
        setting.app_port = 8000;

        let app_state = AppState {
            db,
            setting,
            producer: None,
        };
        let api_call = AssistantPlannedApiCallDTO {
            method: "GET".to_string(),
            path: "/api/v1/user/".to_string(),
            query: json!({
                "search": "Admin User",
                "page": 2,
            }),
            body: json!({}),
        };

        let url = build_internal_url(&app_state, &api_call).unwrap();

        assert!(url.starts_with("http://localhost:8000/api/v1/user/?"));
        assert!(url.contains("search=Admin+User"));
        assert!(url.contains("page=2"));
    }

    #[tokio::test]
    async fn build_internal_url_brackets_ipv6_and_skips_null_query_values() {
        let db = get_db("sqlite::memory:").await.unwrap();
        let mut setting = Setting::new();
        setting.app_host = "::1".to_string();
        setting.app_port = 3000;

        let app_state = AppState {
            db,
            setting,
            producer: None,
        };
        let api_call = AssistantPlannedApiCallDTO {
            method: "GET".to_string(),
            path: "/api/v1/user/profile/".to_string(),
            query: json!({
                "lang": "vi",
                "unused": null,
            }),
            body: json!({}),
        };

        let url = build_internal_url(&app_state, &api_call).unwrap();

        assert!(url.starts_with("http://[::1]:3000/api/v1/user/profile/?"));
        assert!(url.contains("lang=vi"));
        assert!(!url.contains("unused="));
    }
}
