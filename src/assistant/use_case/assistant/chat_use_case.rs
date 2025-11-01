use axum::http::{HeaderMap, StatusCode};
use reqwest::Client;
use rust_i18n::t;

use crate::{
    assistant::{
        dto::assistant_dto::{
            AssistantApiCallDTO, AssistantChatRequestDTO, AssistantChatResponseDTO,
        },
        service::assistant_service,
    },
    config::app::AppState,
    core::dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
};

pub async fn execute(
    app_state: &AppState,
    headers: HeaderMap,
    dto: AssistantChatRequestDTO,
    locale: &str,
) -> Result<ResponseDTO<AssistantChatResponseDTO>, ErrorDTO> {
    assistant_service::validate_message(&dto.message, locale)?;
    assistant_service::validate_context_messages(&dto.messages, locale)?;

    let api_key = app_state.setting.openai_api_key.clone().ok_or_else(|| {
        ErrorDTO::new(
            StatusCode::SERVICE_UNAVAILABLE,
            t!("assistant.config.openai_api_key_missing", locale = locale).to_string(),
        )
    })?;

    let client = Client::new();
    let openapi_json = assistant_service::openapi_json()?;
    let plan = assistant_service::plan_api_call(
        &client,
        &api_key,
        &app_state.setting.openai_model,
        &dto.message,
        &dto.messages,
        &openapi_json,
        locale,
    )
    .await?;

    let Some(api_call) = plan.api_call else {
        let draft_answer = assistant_service::format_no_api_answer(plan.answer.as_deref(), locale);
        let answer = assistant_service::refine_answer_with_openai(
            &client,
            &api_key,
            &app_state.setting.openai_model,
            &dto.message,
            &dto.messages,
            &draft_answer,
            locale,
        )
        .await?;
        return Ok(ResponseDTO::new(
            StatusCode::OK,
            AssistantChatResponseDTO::markdown(answer, None),
        ));
    };

    assistant_service::validate_api_call(&openapi_json, &api_call, locale)?;
    let api_result =
        assistant_service::execute_api_call(app_state, &client, &headers, &api_call, locale)
            .await?;
    let draft_answer = assistant_service::format_api_result_for_user(&api_result, locale);
    let final_answer = assistant_service::refine_api_result_answer_with_openai(
        &client,
        &api_key,
        &app_state.setting.openai_model,
        &dto.message,
        &dto.messages,
        assistant_service::AssistantApiResultRefinement {
            api_call: &api_call,
            api_result: &api_result,
            draft_answer: &draft_answer,
        },
        locale,
    )
    .await?;

    Ok(ResponseDTO::new(
        StatusCode::OK,
        AssistantChatResponseDTO::markdown(
            final_answer,
            Some(AssistantApiCallDTO {
                method: api_call.method.to_uppercase(),
                path: api_call.path,
                status: api_result.status,
            }),
        ),
    ))
}
