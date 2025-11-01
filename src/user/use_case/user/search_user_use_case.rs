use crate::{
    core::{
        context::Context,
        dto::{error_dto::ErrorDTO, response_dto::ResponseDTO},
    },
    user::{
        dto::user_dto::{UserListDTO, UserSearchParamsDTO},
        repository::user_repository::{self, UserOrderBy, UserSearchParams},
        service::user_service,
    },
};
use axum::http::StatusCode;

pub async fn execute(
    context: &Context<'_>,
    dto: UserSearchParamsDTO,
) -> Result<ResponseDTO<UserListDTO>, ErrorDTO> {
    // Parse order_by string into OrderBy structs
    let order_by_list = if let Some(order_by_str) = &dto.order_by {
        UserOrderBy::parse_order_by_string(order_by_str)
    } else {
        Vec::new()
    };

    let users = user_repository::search(
        context,
        &UserSearchParams {
            email: dto.email.as_deref(),
            first_name: dto.first_name.as_deref(),
            last_name: dto.last_name.as_deref(),
            page: dto.page,
            page_size: dto.page_size,
            order_by: if order_by_list.is_empty() {
                None
            } else {
                Some(&order_by_list)
            },
            ..Default::default()
        },
    )
    .await
    .map_err(ErrorDTO::map_internal_error)?;

    let user_dtos = user_service::models_to_dtos(context, &users).await?;
    let total_count = users.len();

    Ok(ResponseDTO::new(
        StatusCode::OK,
        UserListDTO {
            items: user_dtos,
            count: total_count,
        },
    ))
}
