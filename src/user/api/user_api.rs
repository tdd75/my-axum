use crate::config::app::AppState;
use crate::core::db::entity::user;
use crate::core::db::uow::new_transaction;
use crate::core::dto::error_dto::ErrorDTO;
use crate::core::dto::response_dto::ResponseDTO;
use crate::core::dto::util::deserialize_with_fields;
use crate::user::dto::avatar_dto::{UploadAvatarDTO, UploadAvatarResponseDTO};
use crate::user::dto::user_dto::{
    UserCreateDTO, UserDTO, UserListDTO, UserSearchParamsDTO, UserUpdateDTO,
};
use crate::user::use_case::user::{
    create_user_use_case, delete_user_use_case, get_user_use_case, search_user_use_case,
    update_user_use_case, upload_avatar_use_case,
};
use axum::extract::{Path, Query, State};
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
    app_state: State<AppState>,
    Extension(current_user): Extension<user::Model>,
    Query(dto): Query<UserSearchParamsDTO>,
) -> Result<ResponseDTO<UserListDTO>, ErrorDTO> {
    new_transaction(&app_state, Some(current_user), |context| {
        Box::pin(search_user_use_case::execute(context, dto))
    })
    .await
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
    app_state: State<AppState>,
    Extension(current_user): Extension<user::Model>,
    Json(dto): Json<UserCreateDTO>,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    new_transaction(&app_state, Some(current_user), |context| {
        Box::pin(create_user_use_case::execute(context, dto))
    })
    .await
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
    app_state: State<AppState>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<i32>,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    new_transaction(&app_state, Some(current_user), move |context| {
        Box::pin(get_user_use_case::execute(context, id))
    })
    .await
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
    app_state: State<AppState>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<i32>,
    Json(body): Json<Value>,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    let (dto, fields) = deserialize_with_fields(body)?;

    new_transaction(&app_state, Some(current_user), move |context| {
        Box::pin(update_user_use_case::execute(context, id, dto, fields))
    })
    .await
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
    app_state: State<AppState>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<i32>,
) -> Result<ResponseDTO<()>, ErrorDTO> {
    new_transaction(&app_state, Some(current_user), move |context| {
        Box::pin(delete_user_use_case::execute(context, id))
    })
    .await
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
    app_state: State<AppState>,
    Extension(current_user): Extension<user::Model>,
    Json(dto): Json<UploadAvatarDTO>,
) -> Result<ResponseDTO<UploadAvatarResponseDTO>, ErrorDTO> {
    new_transaction(&app_state, Some(current_user), move |context| {
        Box::pin(upload_avatar_use_case::execute(context, dto))
    })
    .await
}
