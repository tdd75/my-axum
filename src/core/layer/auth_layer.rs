use crate::config::app::AppState;
use crate::core::db::uow::new_transaction;
use crate::core::dto::error_dto::ErrorDTO;
use crate::user::service::auth_service::{self, TokenType};
use axum::extract::State;
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ErrorDTO> {
    let headers = req.headers().clone();

    let uri_query = req.uri().query().map(|q| q.to_string());

    let current_user = new_transaction(&app_state, None, move |context| {
        Box::pin(async move {
            let access_token =
                auth_service::extract_token_from_header_or_cookie(&headers, TokenType::Access)
                    .await
                    .or_else(|_| {
                        // Try to get token from query parameters (for WebSocket connections)
                        uri_query
                            .as_ref()
                            .and_then(|query| {
                                auth_service::get_token_from_query_params(query, "token")
                            })
                            .ok_or_else(|| {
                                ErrorDTO::new(
                                    axum::http::StatusCode::UNAUTHORIZED,
                                    "Access token not found".to_string(),
                                )
                            })
                    })?;

            auth_service::get_current_user(context, &access_token).await
        })
    })
    .await?;

    req.extensions_mut().insert(current_user);

    Ok(next.run(req).await)
}
