use crate::core::context::Context;
use crate::core::dto::error_dto::ErrorDTO;
use crate::core::dto::response_dto::ResponseDTO;
use crate::core::dto::util::deserialize_with_fields;
use crate::core::layer::auth_layer::authorize_role;
use crate::user::dto::auth_dto::{ProfileDTO, UpdateProfileDTO};
use crate::user::dto::avatar_dto::{UploadAvatarDTO, UploadAvatarResponseDTO};
use crate::user::dto::user_dto::{
    UserCreateDTO, UserDTO, UserListDTO, UserSearchParamsDTO, UserUpdateDTO,
};
use crate::user::entity::{sea_orm_active_enums::UserRole, user};
use crate::user::use_case::auth::{get_profile_use_case, update_profile_use_case};
use crate::user::use_case::user::{
    create_user_use_case, delete_user_use_case, get_user_use_case, search_user_use_case,
    update_user_use_case, upload_avatar_use_case,
};
use axum::extract::{Path, Query};
#[allow(unused_imports)]
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde_json::Value;

#[utoipa::path(
    get,
    path = "/api/v1/user/",
    tags = ["User"],
    security(("bearer_auth" = [])),
    params(UserSearchParamsDTO),
    responses((status = StatusCode::OK, body = UserListDTO)),
)]
pub async fn search_user(
    Extension(current_user): Extension<user::Model>,
    Extension(context): Extension<Context>,
    Query(dto): Query<UserSearchParamsDTO>,
) -> Result<ResponseDTO<UserListDTO>, ErrorDTO> {
    authorize_role(&context, &current_user, UserRole::Admin)?;

    search_user_use_case::execute(&context, dto).await
}

#[utoipa::path(
    post,
    path = "/api/v1/user/",
    tags = ["User"],
    security(("bearer_auth" = [])),
    request_body(content = UserCreateDTO),
    responses((status = StatusCode::CREATED, body = UserDTO)),
)]
pub async fn create_user(
    Extension(current_user): Extension<user::Model>,
    Extension(context): Extension<Context>,
    Json(dto): Json<UserCreateDTO>,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    authorize_role(&context, &current_user, UserRole::Admin)?;

    create_user_use_case::execute(&context, dto).await
}

#[utoipa::path(
    get,
    path = "/api/v1/user/{id}/",
    tags = ["User"],
    security(("bearer_auth" = [])),
    params(("id" = i32, Path)),
    responses((status = StatusCode::OK, body = UserDTO)),
)]
pub async fn get_user(
    Extension(current_user): Extension<user::Model>,
    Extension(context): Extension<Context>,
    Path(id): Path<i32>,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    authorize_role(&context, &current_user, UserRole::Admin)?;

    get_user_use_case::execute(&context, id).await
}

#[utoipa::path(
    patch,
    path = "/api/v1/user/{id}/",
    tags = ["User"],
    security(("bearer_auth" = [])),
    params(("id" = i32, Path)),
    request_body(content = UserUpdateDTO),
    responses((status = StatusCode::OK, body = UserDTO)),
)]
pub async fn update_user(
    Extension(current_user): Extension<user::Model>,
    Extension(context): Extension<Context>,
    Path(id): Path<i32>,
    Json(body): Json<Value>,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    authorize_role(&context, &current_user, UserRole::Admin)?;

    let (dto, fields) = deserialize_with_fields(body, &context.locale)?;
    update_user_use_case::execute(&context, id, dto, fields).await
}

#[utoipa::path(
    delete,
    path = "/api/v1/user/{id}/",
    tags = ["User"],
    security(("bearer_auth" = [])),
    params(("id" = i32, Path)),
    responses((status = StatusCode::NO_CONTENT)),
)]
pub async fn delete_user(
    Extension(current_user): Extension<user::Model>,
    Extension(context): Extension<Context>,
    Path(id): Path<i32>,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    authorize_role(&context, &current_user, UserRole::Admin)?;

    delete_user_use_case::execute(&context, id).await
}

#[utoipa::path(
    get,
    path = "/api/v1/user/profile/",
    tags = ["User"],
    security(("bearer_auth" = [])),
    responses((status = StatusCode::OK, body = ProfileDTO)),
)]
pub async fn get_profile(
    Extension(context): Extension<Context>,
) -> Result<ResponseDTO<ProfileDTO>, ErrorDTO> {
    get_profile_use_case::execute(&context).await
}

#[utoipa::path(
    patch,
    path = "/api/v1/user/profile/",
    tags = ["User"],
    security(("bearer_auth" = [])),
    request_body(content = UpdateProfileDTO),
    responses((status = StatusCode::OK, body = ProfileDTO)),
)]
pub async fn update_profile(
    Extension(context): Extension<Context>,
    Json(body): Json<Value>,
) -> Result<ResponseDTO<ProfileDTO>, ErrorDTO> {
    let (dto, fields) = deserialize_with_fields(body, &context.locale)?;
    update_profile_use_case::execute(&context, dto, fields).await
}

#[utoipa::path(
    post,
    path = "/api/v1/user/upload-avatar/",
    tags = ["User"],
    security(("bearer_auth" = [])),
    request_body(content = UploadAvatarDTO),
    responses((status = StatusCode::ACCEPTED, body = UploadAvatarResponseDTO)),
)]
pub async fn upload_avatar(
    Extension(context): Extension<Context>,
    Json(dto): Json<UploadAvatarDTO>,
) -> Result<ResponseDTO<UploadAvatarResponseDTO>, ErrorDTO> {
    upload_avatar_use_case::execute(&context, dto, &context.locale).await
}
