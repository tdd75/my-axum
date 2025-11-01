use axum::http::HeaderMap;
use axum::{Extension, Json};

use crate::core::context::Context;
use crate::{
    core::dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    user::{
        dto::auth_dto::{
            ChangePasswordDTO, ForgotPasswordDTO, LoginDTO, RefreshTokenDTO, RegisterDTO,
            ResetPasswordDTO, TokenPairDTO,
        },
        use_case::auth::{
            change_password_use_case, forgot_password_use_case, login_use_case, logout_use_case,
            refresh_token_use_case, register_use_case, reset_password_use_case,
        },
    },
};

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
    Extension(context): Extension<Context>,
    headers: HeaderMap,
    Json(dto): Json<LoginDTO>,
) -> Result<ResponseDTO<TokenPairDTO>, ErrorDTO> {
    login_use_case::execute(&context, dto, headers).await
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
    Extension(context): Extension<Context>,
    headers: HeaderMap,
    Json(dto): Json<RegisterDTO>,
) -> Result<ResponseDTO<TokenPairDTO>, ErrorDTO> {
    register_use_case::execute(&context, dto, headers).await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout/",
    tags = ["Auth"],
    security(("bearer_auth" = [])),
    responses((status = 204)),
)]
pub async fn logout(
    Extension(context): Extension<Context>,
    headers: HeaderMap,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    logout_use_case::execute(&context, headers).await
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh-token/",
    tags = ["Auth"],
    request_body(content = RefreshTokenDTO),
    responses((status = 200, body = TokenPairDTO)),
)]
pub async fn refresh_token(
    Extension(context): Extension<Context>,
    headers: HeaderMap,
    Json(dto): Json<RefreshTokenDTO>,
) -> Result<ResponseDTO<TokenPairDTO>, ErrorDTO> {
    refresh_token_use_case::execute(&context, dto, headers).await
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
    Extension(context): Extension<Context>,
    Json(dto): Json<ForgotPasswordDTO>,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    forgot_password_use_case::execute(&context, dto).await
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
    Extension(context): Extension<Context>,
    Json(dto): Json<ResetPasswordDTO>,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    reset_password_use_case::execute(&context, dto).await
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
    Extension(context): Extension<Context>,
    Json(dto): Json<ChangePasswordDTO>,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    change_password_use_case::execute(&context, dto).await
}
