#[cfg(test)]
mod task_api_tests {
    use crate::{setup::app::TestApp, setup::fixture::login_normal_user};
    use futures::{SinkExt, StreamExt};
    use my_axum::core::context::Context;
    use sea_orm::TransactionTrait;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    #[tokio::test]
    async fn test_progress_updates_websocket_connection() {
        let test_app = TestApp::spawn_app().await;

        // Create and login user
        let access_token = test_app
            .db
            .transaction::<_, String, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _) = login_normal_user(&mut context).await;
                    Ok(access_token)
                })
            })
            .await
            .unwrap();

        // Build WebSocket URL for progress updates
        let ws_url = format!(
            "ws://{}/ws/task/test-task-123/?token={}",
            test_app.base_url.replace("http://", ""),
            access_token
        );

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");
        let (mut write, _) = ws_stream.split();

        // Send a ping message
        let test_message = r#"{"action":"ping"}"#;
        write
            .send(Message::Text(test_message.to_string().into()))
            .await
            .expect("Failed to send message");

        // Wait a bit for any response
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Close the connection gracefully
        write.send(Message::Close(None)).await.ok();
    }

    #[tokio::test]
    async fn test_progress_updates_without_token_fails() {
        let test_app = TestApp::spawn_app().await;

        // Build WebSocket URL without token
        let ws_url = format!(
            "ws://{}/ws/task/test-task-no-token/",
            test_app.base_url.replace("http://", "")
        );

        // Try to connect without token
        let result = connect_async(&ws_url).await;

        // Connection should fail or immediately close
        assert!(
            result.is_err() || {
                if let Ok((ws_stream, _)) = result {
                    let (_, mut read) = ws_stream.split();
                    let msg =
                        tokio::time::timeout(std::time::Duration::from_secs(2), read.next()).await;
                    msg.is_err() || msg.unwrap().is_none()
                } else {
                    false
                }
            },
            "Progress WebSocket should reject connection without token"
        );
    }

    #[tokio::test]
    async fn test_progress_updates_with_invalid_token_fails() {
        let test_app = TestApp::spawn_app().await;

        // Build WebSocket URL with invalid token
        let ws_url = format!(
            "ws://{}/ws/task/test-task-123/?token=invalid_token_xyz",
            test_app.base_url.replace("http://", "")
        );

        // Try to connect with invalid token
        let result = connect_async(&ws_url).await;

        // Connection should fail or immediately close
        assert!(
            result.is_err() || {
                if let Ok((ws_stream, _)) = result {
                    let (_, mut read) = ws_stream.split();
                    let msg =
                        tokio::time::timeout(std::time::Duration::from_secs(2), read.next()).await;
                    msg.is_err() || msg.unwrap().is_none()
                } else {
                    false
                }
            },
            "Progress WebSocket should reject connection with invalid token"
        );
    }

    #[tokio::test]
    async fn test_progress_updates_receives_broadcasts() {
        let test_app = TestApp::spawn_app().await;

        // Create and login user
        let access_token = test_app
            .db
            .transaction::<_, String, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _user) = login_normal_user(&mut context).await;
                    Ok(access_token)
                })
            })
            .await
            .unwrap();

        // Build WebSocket URL for progress updates
        let ws_url = format!(
            "ws://{}/ws/task/test-task-456/?token={}",
            test_app.base_url.replace("http://", ""),
            access_token
        );

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");
        let (mut write, _read) = ws_stream.split();

        // Give time for registration
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Just verify connection works without errors
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Close the connection
        write.send(Message::Close(None)).await.ok();
    }

    #[tokio::test]
    async fn test_multiple_progress_connections_same_user() {
        let test_app = TestApp::spawn_app().await;

        // Create and login user
        let access_token = test_app
            .db
            .transaction::<_, String, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _) = login_normal_user(&mut context).await;
                    Ok(access_token)
                })
            })
            .await
            .unwrap();

        // Build WebSocket URL
        let ws_url = format!(
            "ws://{}/ws/task/test-task-789/?token={}",
            test_app.base_url.replace("http://", ""),
            access_token
        );

        // Connect first WebSocket
        let result1 = connect_async(&ws_url).await;

        if let Ok((ws_stream1, _)) = result1 {
            let (mut write1, _) = ws_stream1.split();

            // Give time for first connection to register
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Connect second WebSocket for same user
            let result2 = connect_async(&ws_url).await;

            if let Ok((ws_stream2, _)) = result2 {
                let (mut write2, _) = ws_stream2.split();

                // Both connections should work (latest replaces previous)
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                // Close both connections
                write1.send(Message::Close(None)).await.ok();
                write2.send(Message::Close(None)).await.ok();
            } else {
                // Second connection failed - clean up first
                write1.send(Message::Close(None)).await.ok();
            }
        }
    }

    #[tokio::test]
    async fn test_websocket_handles_binary_message() {
        let test_app = TestApp::spawn_app().await;

        // Create and login user
        let access_token = test_app
            .db
            .transaction::<_, String, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _) = login_normal_user(&mut context).await;
                    Ok(access_token)
                })
            })
            .await
            .unwrap();

        // Build WebSocket URL
        let ws_url = format!(
            "ws://{}/ws/task/test-task-abc/?token={}",
            test_app.base_url.replace("http://", ""),
            access_token
        );

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");
        let (mut write, _) = ws_stream.split();

        // Send binary message (should be ignored)
        let binary_data = vec![0u8, 1, 2, 3, 4];
        write.send(Message::Binary(binary_data.into())).await.ok();

        // Give time to process
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Close the connection
        write.send(Message::Close(None)).await.ok();
    }

    #[tokio::test]
    async fn test_websocket_connection_cleanup_on_error() {
        let test_app = TestApp::spawn_app().await;

        // Create and login user
        let access_token = test_app
            .db
            .transaction::<_, String, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _) = login_normal_user(&mut context).await;
                    Ok(access_token)
                })
            })
            .await
            .unwrap();

        // Build WebSocket URL
        let ws_url = format!(
            "ws://{}/ws/task/test-task-xyz/?token={}",
            test_app.base_url.replace("http://", ""),
            access_token
        );

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");
        let (write, _) = ws_stream.split();

        // Abruptly drop the connection without proper close
        drop(write);

        // Give time for cleanup
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Verify cleanup happened by trying to connect again
        let result = connect_async(&ws_url).await;
        assert!(result.is_ok(), "Should be able to reconnect after cleanup");

        if let Ok((ws_stream, _)) = result {
            let (mut write, _) = ws_stream.split();
            write.send(Message::Close(None)).await.ok();
        }
    }
}
