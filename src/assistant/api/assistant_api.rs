use axum::{Extension, Json, extract::State, http::HeaderMap};

use crate::{
    assistant::{
        dto::assistant_dto::{AssistantChatRequestDTO, AssistantChatResponseDTO},
        use_case::assistant::chat_use_case,
    },
    config::app::AppState,
    core::{
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
        layer::lang_layer::RequestLocale,
    },
};

#[utoipa::path(
    post,
    path = "/api/v1/assistant/chat/",
    tags = ["Assistant"],
    security(("bearer_auth" = [])),
    request_body(
        content = AssistantChatRequestDTO,
        example = json!({ "message": "Show my profile" }),
    ),
    responses((status = 200, body = AssistantChatResponseDTO)),
)]
pub async fn chat(
    State(app_state): State<AppState>,
    Extension(locale): Extension<RequestLocale>,
    headers: HeaderMap,
    Json(dto): Json<AssistantChatRequestDTO>,
) -> Result<ResponseDTO<AssistantChatResponseDTO>, ErrorDTO> {
    chat_use_case::execute(&app_state, headers, dto, locale.as_str()).await
}
