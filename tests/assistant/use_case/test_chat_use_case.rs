use axum::http::{HeaderMap, StatusCode};
use my_axum::{
    assistant::{
        dto::assistant_dto::{
            AssistantChatMessageDTO, AssistantChatMessageRoleDTO, AssistantChatRequestDTO,
        },
        use_case::assistant::chat_use_case,
    },
    config::{app::AppState, setting::Setting},
    core::db::connection::get_db,
};

async fn test_state_without_openai_key() -> AppState {
    let db = get_db("sqlite::memory:").await.unwrap();
    let mut setting = Setting::new();
    setting.openai_api_key = None;

    AppState {
        db,
        setting,
        producer: None,
    }
}

#[tokio::test]
async fn execute_rejects_empty_message_before_key_check() {
    let app_state = test_state_without_openai_key().await;
    let dto = AssistantChatRequestDTO {
        message: "   ".to_string(),
        messages: vec![],
    };

    let error = chat_use_case::execute(&app_state, HeaderMap::new(), dto, "en")
        .await
        .unwrap_err();
    assert_eq!(error.status, StatusCode::BAD_REQUEST);
    assert_eq!(error.message, "Message is required");
}

#[tokio::test]
async fn execute_rejects_too_long_message_before_key_check() {
    let app_state = test_state_without_openai_key().await;
    let dto = AssistantChatRequestDTO {
        message: "a".repeat(2_001),
        messages: vec![],
    };

    let error = chat_use_case::execute(&app_state, HeaderMap::new(), dto, "en")
        .await
        .unwrap_err();
    assert_eq!(error.status, StatusCode::BAD_REQUEST);
    assert!(
        error
            .message
            .contains("Message must be at most 2000 characters")
    );
}

#[tokio::test]
async fn execute_rejects_too_many_context_messages_before_key_check() {
    let app_state = test_state_without_openai_key().await;
    let messages = (0..13)
        .map(|index| AssistantChatMessageDTO {
            role: AssistantChatMessageRoleDTO::User,
            content: format!("message {index}"),
        })
        .collect::<Vec<_>>();

    let dto = AssistantChatRequestDTO {
        message: "show my profile".to_string(),
        messages,
    };

    let error = chat_use_case::execute(&app_state, HeaderMap::new(), dto, "en")
        .await
        .unwrap_err();
    assert_eq!(error.status, StatusCode::BAD_REQUEST);
    assert!(
        error
            .message
            .contains("Conversation context must include at most 12 messages")
    );
}

#[tokio::test]
async fn execute_rejects_too_long_context_message_before_key_check() {
    let app_state = test_state_without_openai_key().await;
    let dto = AssistantChatRequestDTO {
        message: "show my profile".to_string(),
        messages: vec![AssistantChatMessageDTO {
            role: AssistantChatMessageRoleDTO::Assistant,
            content: "a".repeat(2_001),
        }],
    };

    let error = chat_use_case::execute(&app_state, HeaderMap::new(), dto, "en")
        .await
        .unwrap_err();
    assert_eq!(error.status, StatusCode::BAD_REQUEST);
    assert!(
        error
            .message
            .contains("Each context message must be at most 2000 characters")
    );
}

#[tokio::test]
async fn execute_returns_service_unavailable_when_openai_key_missing() {
    let app_state = test_state_without_openai_key().await;
    let dto = AssistantChatRequestDTO {
        message: "show my profile".to_string(),
        messages: vec![],
    };

    let error = chat_use_case::execute(&app_state, HeaderMap::new(), dto, "en")
        .await
        .unwrap_err();
    assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error.message, "OPENAI_API_KEY is not configured");
}
