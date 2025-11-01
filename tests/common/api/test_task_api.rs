#[cfg(test)]
mod task_api_tests {
    use crate::{setup::app::TestApp, setup::fixture::login_normal_user};
    use futures::{SinkExt, StreamExt};
    use my_axum::{
        core::context::Context,
        pkg::broadcast::websocket::{BroadcastMessage, broadcast_to_task},
    };
    use sea_orm::TransactionTrait;
    use serde_json::json;
    use std::{
        sync::{Arc, OnceLock},
        time::Duration,
    };
    use tokio::sync::Mutex;
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use uuid::Uuid;

    fn ws_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    async fn login_and_get_access_token(test_app: &TestApp) -> String {
        test_app
            .db
            .transaction::<_, String, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context::builder(Arc::new(txn.begin().await?)).build();
                    let (access_token, _) = login_normal_user(&mut context).await;
                    context.commit().await?;
                    Ok(access_token)
                })
            })
            .await
            .unwrap()
    }

    fn task_ws_url(test_app: &TestApp, task_id: &str, access_token: &str) -> String {
        format!(
            "ws://{}/ws/v1/task/{task_id}/?token={access_token}",
            test_app.base_url.replace("http://", "")
        )
    }

    #[tokio::test]
    async fn test_progress_updates_websocket_receives_broadcasts() {
        let _guard = ws_test_lock().lock().await;
        let test_app = TestApp::spawn_app().await;
        let access_token = login_and_get_access_token(&test_app).await;
        let task_id = format!("progress-task-{}", Uuid::new_v4());
        let ws_url = task_ws_url(&test_app, &task_id, &access_token);

        let (ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");
        let (mut write, mut read) = ws_stream.split();

        write
            .send(Message::Text(r#"{"action":"ping"}"#.to_string().into()))
            .await
            .expect("Failed to send keep-alive message");

        tokio::time::sleep(Duration::from_millis(150)).await;

        broadcast_to_task(
            &task_id,
            BroadcastMessage {
                event_type: "progress".to_string(),
                data: json!({
                    "task_id": task_id,
                    "percent": 50,
                    "stage": "processing"
                }),
            },
        )
        .await;

        let response = tokio::time::timeout(Duration::from_secs(5), read.next())
            .await
            .expect("Timed out waiting for broadcast")
            .expect("WebSocket closed before receiving broadcast")
            .expect("Failed to read WebSocket message");

        let Message::Text(text) = response else {
            panic!("Expected text frame from progress broadcast");
        };

        let payload: BroadcastMessage =
            serde_json::from_str(text.as_ref()).expect("Broadcast payload should be valid JSON");
        assert_eq!(payload.event_type, "progress");
        assert_eq!(payload.data["percent"], 50);
        assert_eq!(payload.data["stage"], "processing");

        write.send(Message::Close(None)).await.ok();
    }

    #[tokio::test]
    async fn test_progress_updates_websocket_handles_binary_messages_and_reconnects() {
        let _guard = ws_test_lock().lock().await;
        let test_app = TestApp::spawn_app().await;
        let access_token = login_and_get_access_token(&test_app).await;
        let task_id = format!("binary-task-{}", Uuid::new_v4());
        let ws_url = task_ws_url(&test_app, &task_id, &access_token);

        let (first_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");
        let (mut first_write, _) = first_stream.split();

        first_write
            .send(Message::Binary(vec![0_u8, 1, 2, 3].into()))
            .await
            .expect("Failed to send binary frame");
        first_write.send(Message::Close(None)).await.ok();

        tokio::time::sleep(Duration::from_millis(200)).await;

        let (second_stream, _) = connect_async(&ws_url).await.expect("Failed to reconnect");
        let (mut second_write, mut second_read) = second_stream.split();

        tokio::time::sleep(Duration::from_millis(150)).await;

        broadcast_to_task(
            &task_id,
            BroadcastMessage {
                event_type: "reconnected".to_string(),
                data: json!({
                    "task_id": task_id,
                    "status": "ready"
                }),
            },
        )
        .await;

        let response = tokio::time::timeout(Duration::from_secs(5), second_read.next())
            .await
            .expect("Timed out waiting for broadcast after reconnect")
            .expect("WebSocket closed before reconnect broadcast")
            .expect("Failed to read reconnect broadcast");

        let Message::Text(text) = response else {
            panic!("Expected text frame after reconnect");
        };

        let payload: BroadcastMessage =
            serde_json::from_str(text.as_ref()).expect("Reconnect payload should be valid JSON");
        assert_eq!(payload.event_type, "reconnected");
        assert_eq!(payload.data["status"], "ready");

        second_write.send(Message::Close(None)).await.ok();
    }

    #[tokio::test]
    async fn test_progress_updates_without_token_fails() {
        let _guard = ws_test_lock().lock().await;
        let test_app = TestApp::spawn_app().await;
        let task_id = format!("unauthorized-task-{}", Uuid::new_v4());
        let ws_url = format!(
            "ws://{}/ws/v1/task/{task_id}/",
            test_app.base_url.replace("http://", "")
        );

        let result = connect_async(&ws_url).await;

        assert!(
            result.is_err() || {
                if let Ok((ws_stream, _)) = result {
                    let (_, mut read) = ws_stream.split();
                    let msg = tokio::time::timeout(Duration::from_secs(2), read.next()).await;
                    msg.is_err() || msg.unwrap().is_none()
                } else {
                    false
                }
            },
            "Progress WebSocket should reject unauthenticated connections"
        );
    }
}
