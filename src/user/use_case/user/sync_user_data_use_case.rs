use crate::config::app::AppState;
use crate::core::db::entity::user;
use crate::core::db::uow::new_transaction;
use crate::core::dto::error_dto::ErrorDTO;
use crate::core::dto::response_dto::ResponseDTO;
use crate::core::dto::util::ToJson;
use crate::user::dto::user_dto::UserDTO;
use crate::user::repository::user_repository;
use crate::user::service::user_service;
use axum::extract::ws::Message;
use axum::extract::ws::Utf8Bytes;
use axum::extract::ws::WebSocket;
#[allow(unused_imports)]
use axum::http::StatusCode;

pub async fn execute(mut socket: WebSocket, app_state: AppState, current_user: user::Model) {
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                tracing::info!("Received from client: {}", text);

                let response_text: String =
                    match fetch_user_data(app_state.clone(), current_user.clone(), text).await {
                        Ok(response_dto) => response_dto.to_json_string(),
                        Err(error_dto) => error_dto.to_json_string(),
                    };

                if socket
                    .send(Message::Text(response_text.into()))
                    .await
                    .is_err()
                {
                    tracing::warn!("Client disconnected while sending");
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("Client closed connection");
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    tracing::info!("WebSocket disconnected: user_id={}", current_user.id);
}

pub async fn fetch_user_data(
    app_state: AppState,
    current_user: user::Model,
    user_id: Utf8Bytes,
) -> Result<ResponseDTO<UserDTO>, ErrorDTO> {
    let user_id: i32 = match user_id.parse() {
        Ok(id) => id,
        Err(_) => {
            return Err(ErrorDTO::new(
                StatusCode::BAD_REQUEST,
                "Invalid user ID format".to_string(),
            ));
        }
    };

    new_transaction(&app_state, Some(current_user.clone()), move |context| {
        Box::pin(async move {
            let user = user_repository::find_by_id(context, user_id)
                .await
                .map_err(ErrorDTO::map_internal_error);

            match user {
                Ok(Some(user_model)) => {
                    let user_dto = user_service::model_to_dto(context, &user_model).await?;
                    Ok(ResponseDTO::new(StatusCode::OK, user_dto))
                }
                Ok(None) => Err(ErrorDTO::new(
                    StatusCode::NOT_FOUND,
                    "User not found".to_string(),
                )),
                Err(e) => Err(e),
            }
        })
    })
    .await
    .map_err(|e: ErrorDTO| {
        tracing::error!("Transaction error: {:?}", e);
        e
    })
}
