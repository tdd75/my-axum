use axum::http::HeaderMap;
use axum::{Extension, Json, extract::State};
use serde_json::Value;

use crate::config::app::AppState;
use crate::core::db::uow::new_transaction;
use crate::core::dto::util::deserialize_with_fields;
use crate::{
    core::{
        db::entity::user,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    user::{
        dto::auth_dto::{
            ChangePasswordDTO, ForgotPasswordDTO, LoginDTO, ProfileDTO, RefreshTokenDTO,
            RegisterDTO, ResetPasswordDTO, TokenPairDTO, UpdateProfileDTO,
        },
        use_case::auth::{
            change_password_use_case, forgot_password_use_case, get_profile_use_case,
            login_use_case, logout_use_case, refresh_token_use_case, register_use_case,
            reset_password_use_case, update_profile_use_case,
        },
    },
};

#[utoipa::path(
    get,
    path = "/api/v1/auth/me/",
    tags = ["Auth"],
    security(("bearer_auth" = [])),
    responses((status = 200, body = ProfileDTO)),
)]
pub async fn get_profile(
    app_state: State<AppState>,
    Extension(current_user): Extension<user::Model>,
) -> Result<ResponseDTO<ProfileDTO>, ErrorDTO> {
    new_transaction(&app_state, Some(current_user.clone()), |context| {
        Box::pin(get_profile_use_case::execute(context))
    })
    .await
}

#[utoipa::path(
    patch,
    path = "/api/v1/auth/me/",
    tags = ["Auth"],
    security(("bearer_auth" = [])),
    request_body(content = UpdateProfileDTO),
    responses((status = 200, body = ProfileDTO)),
)]
pub async fn update_profile(
    app_state: State<AppState>,
    Extension(current_user): Extension<user::Model>,
    Json(body): Json<Value>,
) -> Result<ResponseDTO<ProfileDTO>, ErrorDTO> {
    let (dto, fields) = deserialize_with_fields(body)?;

    new_transaction(&app_state, Some(current_user.clone()), move |context| {
        Box::pin(update_profile_use_case::execute(context, dto, fields))
    })
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/change-password/",
    tags = ["Auth"],
    security(("bearer_auth" = [])),
    request_body(content = ChangePasswordDTO),
    responses((status = 204)),
)]
pub async fn change_password(
    app_state: State<AppState>,
    Extension(current_user): Extension<user::Model>,
    Json(dto): Json<ChangePasswordDTO>,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    new_transaction(&app_state, Some(current_user.clone()), |context| {
        Box::pin(change_password_use_case::execute(context, dto))
    })
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login/",
    tags = ["Auth"],
    request_body(
        content = LoginDTO,
        example = json!({ "email": "user@example.com", "password": "password123@" }),
    ),
    responses((status = 200, body = TokenPairDTO)),
)]
pub async fn login(
    app_state: State<AppState>,
    headers: HeaderMap,
    Json(dto): Json<LoginDTO>,
) -> Result<ResponseDTO<TokenPairDTO>, ErrorDTO> {
    new_transaction(&app_state, None, |context| {
        Box::pin(login_use_case::execute(context, dto, headers))
    })
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register/",
    tags = ["Auth"],
    request_body(
        content = RegisterDTO,
        example = json!({ "email": "user@example.com", "password": "password123@", "first_name": "John", "last_name": "Doe" }),
    ),
    responses((status = 200, body = TokenPairDTO)),
)]
pub async fn register(
    app_state: State<AppState>,
    headers: HeaderMap,
    Json(dto): Json<RegisterDTO>,
) -> Result<ResponseDTO<TokenPairDTO>, ErrorDTO> {
    new_transaction(&app_state, None, |context| {
        Box::pin(register_use_case::execute(context, dto, headers))
    })
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh-token/",
    tags = ["Auth"],
    request_body(content = RefreshTokenDTO),
    responses((status = 200, body = TokenPairDTO)),
)]
pub async fn refresh_token(
    app_state: State<AppState>,
    headers: HeaderMap,
    Json(dto): Json<RefreshTokenDTO>,
) -> Result<ResponseDTO<TokenPairDTO>, ErrorDTO> {
    new_transaction(&app_state, None, |context| {
        Box::pin(refresh_token_use_case::execute(context, dto, headers))
    })
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout/",
    tags = ["Auth"],
    security(("bearer_auth" = [])),
    responses((status = 204)),
)]
pub async fn logout(
    app_state: State<AppState>,
    headers: HeaderMap,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    new_transaction(&app_state, None, |context| {
        Box::pin(logout_use_case::execute(context, headers))
    })
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/forgot-password/",
    tags = ["Auth"],
    request_body(
        content = ForgotPasswordDTO,
        example = json!({ "email": "user@example.com" }),
    ),
    responses((status = 204)),
)]
pub async fn forgot_password(
    app_state: State<AppState>,
    Json(dto): Json<ForgotPasswordDTO>,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    new_transaction(&app_state, None, |context| {
        Box::pin(forgot_password_use_case::execute(context, dto))
    })
    .await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/reset-password/",
    tags = ["Auth"],
    request_body(
        content = ResetPasswordDTO,
        example = json!({ "email": "user@example.com", "otp": "123456", "new_password": "newpassword123@" }),
    ),
    responses((status = 204)),
)]
pub async fn reset_password(
    app_state: State<AppState>,
    Json(dto): Json<ResetPasswordDTO>,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    new_transaction(&app_state, None, |context| {
        Box::pin(reset_password_use_case::execute(context, dto))
    })
    .await
}
