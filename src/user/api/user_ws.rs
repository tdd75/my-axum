use crate::config::app::AppState;
use crate::core::db::entity::user;
use crate::user::use_case::user::sync_user_data_use_case;
#[allow(unused_imports)]
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    Extension,
    extract::{State, WebSocketUpgrade},
};

pub async fn sync_user_data(
    ws: WebSocketUpgrade,
    app_state: State<AppState>,
    Extension(current_user): Extension<user::Model>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| sync_user_data_use_case::execute(socket, app_state.0, current_user))
}
