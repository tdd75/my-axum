use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::sync::mpsc;

use crate::config::setting::Setting;
use crate::core::db::entity::user;
use crate::pkg::broadcast::websocket::{
    BroadcastMessage, register_task_websocket, unregister_task_websocket,
};
use crate::pkg::cache::get_cached_task_status;

pub async fn execute(socket: WebSocket, task_id: String, current_user: user::Model) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Register this websocket connection for the task
    register_task_websocket(task_id.clone(), tx).await;

    tracing::info!(
        "Progress updates websocket connected for task_id: {} (user_id: {})",
        task_id,
        current_user.id
    );

    // Try to retrieve cached task status from Redis
    let setting = Setting::new();
    match get_cached_task_status(&setting.redis_url, &task_id).await {
        Ok(Some(cached_data)) => {
            // Send cached status immediately to the client
            let cached_msg = BroadcastMessage {
                event_type: "avatar_upload_progress".to_string(),
                data: cached_data,
            };

            if let Ok(json_str) = serde_json::to_string(&cached_msg)
                && sender.send(Message::Text(json_str.into())).await.is_ok()
            {
                tracing::info!("Sent cached task status to client for task_id: {}", task_id);
            }
        }
        Ok(None) => {
            tracing::debug!("No cached status found for task_id: {}", task_id);
        }
        Err(e) => {
            tracing::warn!(
                "Failed to retrieve cached task status for task_id {}: {}",
                task_id,
                e
            );
        }
    }

    // Spawn a task to send messages from the channel to the websocket
    let task_id_clone = task_id.clone();
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages (mostly for keep-alive)
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                tracing::debug!("Received ping for task {}: {}", task_id, text);
            }
            Ok(Message::Close(_)) => {
                tracing::info!(
                    "Client closed progress updates connection: task_id={}",
                    task_id
                );
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error for task {}: {}", task_id, e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    send_task.abort();
    unregister_task_websocket(task_id_clone).await;
    tracing::info!(
        "Progress updates websocket disconnected: task_id={}",
        task_id
    );
}
