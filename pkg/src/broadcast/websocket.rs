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

fn user_registry_key(user_id: i32) -> String {
    format!("user-{}", user_id)
}

fn serialize_message(message: &BroadcastMessage) -> Option<String> {
    match serde_json::to_string(message) {
        Ok(message) => Some(message),
        Err(error) => {
            tracing::error!("Failed to serialize broadcast message: {}", error);
            None
        }
    }
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
    let Some(tx) = registry.get(task_id) else {
        tracing::debug!("No active websocket for task_id: {}", task_id);
        return;
    };

    let Some(json_str) = serialize_message(&message) else {
        return;
    };

    if let Err(e) = tx.send(Message::Text(json_str.into())) {
        tracing::error!("Failed to send message to task {}: {}", task_id, e);
    } else {
        tracing::debug!("Broadcast message sent to task {}", task_id);
    }
}

/// Broadcast a message to all connected tasks
pub async fn broadcast_to_all(message: BroadcastMessage) {
    let registry = get_registry().read().await;

    let Some(json_str) = serialize_message(&message) else {
        return;
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
    register_task_websocket(user_registry_key(user_id), tx).await;
}

/// Unregister a websocket connection for a user (backward compatibility)
pub async fn unregister_user_websocket(user_id: i32) {
    unregister_task_websocket(user_registry_key(user_id)).await;
}

/// Broadcast a message to a specific user (backward compatibility)
pub async fn broadcast_to_user(user_id: i32, message: BroadcastMessage) {
    broadcast_to_task(&user_registry_key(user_id), message).await;
}

/// Clear all websocket registrations (for testing only)
/// This function should only be used in tests to ensure test isolation
#[doc(hidden)]
pub async fn clear_registry() {
    let mut registry = get_registry().write().await;
    registry.clear();
    tracing::debug!("Cleared websocket registry");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU32, Ordering};

    // Counter for unique task IDs to avoid test interference
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_task_id(prefix: &str) -> String {
        let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("{}-{}", prefix, count)
    }

    #[tokio::test]
    async fn test_register_and_unregister_task_websocket() {
        let task_id = unique_task_id("register-test");
        let (tx, _rx) = mpsc::unbounded_channel();

        register_task_websocket(task_id.clone(), tx).await;

        let registry = get_registry().read().await;
        assert!(registry.contains_key(&task_id));
        drop(registry);

        unregister_task_websocket(task_id.clone()).await;

        let registry = get_registry().read().await;
        assert!(!registry.contains_key(&task_id));
    }

    #[tokio::test]
    async fn test_broadcast_to_task_success() {
        let task_id = unique_task_id("broadcast-success");
        let (tx, mut rx) = mpsc::unbounded_channel();

        register_task_websocket(task_id.clone(), tx).await;

        let message = BroadcastMessage {
            event_type: "progress".to_string(),
            data: json!({"percent": 50}),
        };

        broadcast_to_task(&task_id, message).await;

        let received = rx.recv().await;
        assert!(received.is_some());

        unregister_task_websocket(task_id).await;
    }

    #[tokio::test]
    async fn test_broadcast_to_task_no_connection() {
        let task_id = unique_task_id("no-connection");

        let message = BroadcastMessage {
            event_type: "progress".to_string(),
            data: json!({"percent": 50}),
        };

        // Should not panic when no connection exists
        broadcast_to_task(&task_id, message).await;
    }

    #[tokio::test]
    async fn test_broadcast_to_all() {
        let task_id_a = unique_task_id("broadcast-all-a");
        let task_id_b = unique_task_id("broadcast-all-b");

        let (tx1, mut rx1) = mpsc::unbounded_channel();
        let (tx2, mut rx2) = mpsc::unbounded_channel();

        register_task_websocket(task_id_a.clone(), tx1).await;
        register_task_websocket(task_id_b.clone(), tx2).await;

        let message = BroadcastMessage {
            event_type: "broadcast".to_string(),
            data: json!({"msg": "hello all"}),
        };

        broadcast_to_all(message).await;

        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());

        unregister_task_websocket(task_id_a).await;
        unregister_task_websocket(task_id_b).await;
    }

    #[tokio::test]
    async fn test_user_websocket_compat_functions() {
        let user_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst) as i32 + 10000;
        let expected_key = format!("user-{}", user_id);

        let (tx, _rx) = mpsc::unbounded_channel();
        register_user_websocket(user_id, tx).await;

        let registry = get_registry().read().await;
        assert!(registry.contains_key(&expected_key));
        drop(registry);

        unregister_user_websocket(user_id).await;

        let registry = get_registry().read().await;
        assert!(!registry.contains_key(&expected_key));
    }

    #[tokio::test]
    async fn test_broadcast_to_user() {
        let user_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst) as i32 + 20000;

        let (tx, mut rx) = mpsc::unbounded_channel();
        register_user_websocket(user_id, tx).await;

        let message = BroadcastMessage {
            event_type: "user_event".to_string(),
            data: json!({"user_id": user_id}),
        };

        broadcast_to_user(user_id, message).await;

        let received = rx.recv().await;
        assert!(received.is_some());

        unregister_user_websocket(user_id).await;
    }

    #[test]
    fn test_broadcast_message_serialization() {
        let message = BroadcastMessage {
            event_type: "test".to_string(),
            data: json!({"key": "value"}),
        };

        let json_str = serde_json::to_string(&message).unwrap();
        let parsed: BroadcastMessage = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed.event_type, "test");
        assert_eq!(parsed.data["key"], "value");
    }
}
