use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::{RwLock, mpsc};

/// Message structure for broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub event_type: String,
    pub data: serde_json::Value,
}

/// Type alias for the websocket registry
type WebSocketRegistry = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>;

/// Global registry of task websocket connections
/// Maps task_id to a channel sender for broadcasting messages
static WEBSOCKET_REGISTRY: OnceLock<WebSocketRegistry> = OnceLock::new();

fn get_registry() -> &'static WebSocketRegistry {
    WEBSOCKET_REGISTRY.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

/// Register a websocket connection for a task
pub async fn register_task_websocket(task_id: String, tx: mpsc::UnboundedSender<Message>) {
    let mut registry = get_registry().write().await;
    registry.insert(task_id.clone(), tx);
    tracing::info!("Registered websocket for task_id: {}", task_id);
}

/// Unregister a websocket connection for a task
pub async fn unregister_task_websocket(task_id: String) {
    let mut registry = get_registry().write().await;
    registry.remove(&task_id);
    tracing::info!("Unregistered websocket for task_id: {}", task_id);
}

/// Broadcast a message to a specific task
pub async fn broadcast_to_task(task_id: &str, message: BroadcastMessage) {
    let registry = get_registry().read().await;

    if let Some(tx) = registry.get(task_id) {
        let json_str = match serde_json::to_string(&message) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to serialize broadcast message: {}", e);
                return;
            }
        };

        if let Err(e) = tx.send(Message::Text(json_str.into())) {
            tracing::error!("Failed to send message to task {}: {}", task_id, e);
        } else {
            tracing::debug!("Broadcast message sent to task {}", task_id);
        }
    } else {
        tracing::debug!("No active websocket for task_id: {}", task_id);
    }
}

/// Broadcast a message to all connected tasks
pub async fn broadcast_to_all(message: BroadcastMessage) {
    let registry = get_registry().read().await;

    let json_str = match serde_json::to_string(&message) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to serialize broadcast message: {}", e);
            return;
        }
    };

    for (task_id, tx) in registry.iter() {
        if let Err(e) = tx.send(Message::Text(json_str.clone().into())) {
            tracing::error!("Failed to broadcast to task {}: {}", task_id, e);
        }
    }

    tracing::debug!("Broadcast message sent to {} tasks", registry.len());
}

// Backward compatibility: user_id based functions
/// Register a websocket connection for a user (backward compatibility)
pub async fn register_user_websocket(user_id: i32, tx: mpsc::UnboundedSender<Message>) {
    register_task_websocket(format!("user-{}", user_id), tx).await;
}

/// Unregister a websocket connection for a user (backward compatibility)
pub async fn unregister_user_websocket(user_id: i32) {
    unregister_task_websocket(format!("user-{}", user_id)).await;
}

/// Broadcast a message to a specific user (backward compatibility)
pub async fn broadcast_to_user(user_id: i32, message: BroadcastMessage) {
    broadcast_to_task(&format!("user-{}", user_id), message).await;
}

/// Clear all websocket registrations (for testing only)
/// This function should only be used in tests to ensure test isolation
#[doc(hidden)]
pub async fn clear_registry() {
    let mut registry = get_registry().write().await;
    registry.clear();
    tracing::debug!("Cleared websocket registry");
}
