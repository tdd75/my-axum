use axum::{Extension, Json, extract::State, http::StatusCode};

use crate::{
    config::app::AppState,
    core::{
        context::Context,
        dto::{
            error_dto::ErrorDTO,
            response_dto::ResponseDTO,
            runbook_dto::{
                RunRunbookRequestDTO, RunRunbookResponseDTO, RunbookInfoDTO, RunbookListDTO,
            },
        },
        layer::auth_layer::authorize_role,
        runbook,
    },
    user::entity::{sea_orm_active_enums::UserRole, user},
};

#[utoipa::path(
    get,
    path = "/api/v1/runbook/",
    tags = ["Runbook"],
    security(("bearer_auth" = [])),
    responses((status = 200, body = RunbookListDTO)),
)]
pub async fn list_runbooks(
    Extension(current_user): Extension<user::Model>,
    Extension(context): Extension<Context>,
) -> Result<ResponseDTO<RunbookListDTO>, ErrorDTO> {
    authorize_role(&context, &current_user, UserRole::Admin)?;

    let runbooks = runbook::list()
        .into_iter()
        .map(RunbookInfoDTO::from)
        .collect();

    Ok(ResponseDTO::new(
        StatusCode::OK,
        RunbookListDTO { runbooks },
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/runbook/run/",
    tags = ["Runbook"],
    security(("bearer_auth" = [])),
    request_body(
        content = RunRunbookRequestDTO,
        example = json!({
            "name": "delete-refresh-tokens-by-email",
            "args": ["--email", "user@example.com"]
        }),
    ),
    responses((status = 200, body = RunRunbookResponseDTO)),
)]
pub async fn run_runbook(
    State(app_state): State<AppState>,
    Extension(current_user): Extension<user::Model>,
    Extension(context): Extension<Context>,
    Json(body): Json<RunRunbookRequestDTO>,
) -> Result<ResponseDTO<RunRunbookResponseDTO>, ErrorDTO> {
    authorize_role(&context, &current_user, UserRole::Admin)?;

    let result = runbook::run(&app_state.setting, &body.name, &body.args)
        .await
        .map_err(ErrorDTO::from)?;

    Ok(ResponseDTO::new(
        StatusCode::OK,
        RunRunbookResponseDTO::from(result),
    ))
}
