use crate::common::use_case::task::get_task_progress_use_case;
use crate::core::db::entity::user;
#[allow(unused_imports)]
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    Extension,
    extract::{Path, WebSocketUpgrade},
};

pub async fn get_task_progress(
    ws: WebSocketUpgrade,
    Path(task_id): Path<String>,
    Extension(current_user): Extension<user::Model>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| get_task_progress_use_case::execute(socket, task_id, current_user))
}
