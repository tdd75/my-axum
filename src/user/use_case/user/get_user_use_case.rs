use crate::{
    core::{
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    user::{dto::user_dto::UserDTO, repository::user_repository, service::user_service},
};
use axum::http::StatusCode;
use rust_i18n::t;

pub async fn execute(
    context: &Context<'_>,
    user_id: i32,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    let user = user_repository::find_by_id(context, user_id)
        .await
        .map_err(ErrorDTO::map_internal_error)?;

    match user {
        Some(u) => {
            let user_dto = user_service::model_to_dto(context, &u).await?;

            Ok(ResponseDTO::new(StatusCode::OK, user_dto))
        }
        None => Err(ErrorDTO::new(
            StatusCode::NOT_FOUND,
            t!("user.not_found_with_id", id = user_id).to_string(),
        )),
    }
}
