use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssistantChatRequestDTO {
    #[schema(example = "Show my profile")]
    pub message: String,
    #[serde(default)]
    pub messages: Vec<AssistantChatMessageDTO>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AssistantChatMessageDTO {
    pub role: AssistantChatMessageRoleDTO,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AssistantChatMessageRoleDTO {
    User,
    Assistant,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AssistantApiCallDTO {
    pub method: String,
    pub path: String,
    pub status: u16,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AssistantChatResponseDTO {
    pub message: String,
    pub format: AssistantChatMessageFormatDTO,
    pub api_call: Option<AssistantApiCallDTO>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AssistantChatMessageFormatDTO {
    Markdown,
}

impl AssistantChatResponseDTO {
    pub fn markdown(message: String, api_call: Option<AssistantApiCallDTO>) -> Self {
        Self {
            message,
            format: AssistantChatMessageFormatDTO::Markdown,
            api_call,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AssistantPlanDTO {
    pub answer: Option<String>,
    pub api_call: Option<AssistantPlannedApiCallDTO>,
}

#[derive(Debug, Deserialize)]
pub struct AssistantPlannedApiCallDTO {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub query: Value,
    #[serde(default)]
    pub body: Value,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{AssistantApiCallDTO, AssistantChatResponseDTO};

    #[test]
    fn serializes_chat_response_format_as_markdown() {
        let dto = AssistantChatResponseDTO::markdown(
            "Người dùng có tên **Bench-8-98**.".to_string(),
            None,
        );

        let value = serde_json::to_value(dto).unwrap();

        assert_eq!(value["message"], "Người dùng có tên **Bench-8-98**.");
        assert_eq!(value["format"], "markdown");
        assert_eq!(value["api_call"], serde_json::Value::Null);
    }

    #[test]
    fn serializes_chat_response_api_call_with_markdown_format() {
        let dto = AssistantChatResponseDTO::markdown(
            "- **Email:** user@example.com".to_string(),
            Some(AssistantApiCallDTO {
                method: "GET".to_string(),
                path: "/api/v1/user/".to_string(),
                status: 200,
            }),
        );

        let value = serde_json::to_value(dto).unwrap();

        assert_eq!(value["format"], "markdown");
        assert_eq!(
            value["api_call"],
            json!({
                "method": "GET",
                "path": "/api/v1/user/",
                "status": 200
            })
        );
    }
}
