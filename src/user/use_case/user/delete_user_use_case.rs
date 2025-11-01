use crate::{
    core::{
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    user::repository::user_repository,
};
use axum::http::StatusCode;

pub async fn execute(context: &Context<'_>, user_id: i32) -> Result<ResponseDTO<()>, ErrorDTO> {
    user_repository::delete_by_id(context, user_id)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    Ok(ResponseDTO::new(StatusCode::NO_CONTENT, ()))
}
