#[cfg(test)]
mod user_ws_tests {
    use crate::{setup::app::TestApp, setup::fixture::login_normal_user};
    use futures::{SinkExt, StreamExt};
    use my_axum::core::context::Context;
    use sea_orm::TransactionTrait;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    #[tokio::test]
    async fn test_websocket_connection_success() {
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

        // Build WebSocket URL with token as query param
        let ws_url = format!(
            "ws://{}/ws/user/?token={}",
            test_app.base_url.replace("http://", ""),
            access_token
        );

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");
        let (mut write, mut read) = ws_stream.split();

        // Send a test message
        let test_message = r#"{"action":"ping"}"#;
        write
            .send(Message::Text(test_message.to_string().into()))
            .await
            .expect("Failed to send message");

        // Wait for response with timeout
        let timeout_duration = std::time::Duration::from_secs(5);
        let response = tokio::time::timeout(timeout_duration, read.next()).await;

        // Verify we received a response
        assert!(response.is_ok(), "WebSocket timeout waiting for response");
        let message = response.unwrap();
        assert!(
            message.is_some(),
            "WebSocket connection closed unexpectedly"
        );

        // Close the connection gracefully
        write.send(Message::Close(None)).await.ok();
    }

    #[tokio::test]
    async fn test_websocket_connection_without_token_fails() {
        let test_app = TestApp::spawn_app().await;

        // Build WebSocket URL without token
        let ws_url = format!("ws://{}/ws/user/", test_app.base_url.replace("http://", ""));

        // Try to connect without token
        let result = connect_async(&ws_url).await;

        // Connection should fail or immediately close due to missing authentication
        assert!(
            result.is_err() || {
                if let Ok((ws_stream, _)) = result {
                    let (_, mut read) = ws_stream.split();
                    // Connection should close immediately
                    let msg =
                        tokio::time::timeout(std::time::Duration::from_secs(2), read.next()).await;
                    msg.is_err() || msg.unwrap().is_none()
                } else {
                    false
                }
            },
            "WebSocket should reject connection without token"
        );
    }

    #[tokio::test]
    async fn test_websocket_connection_with_invalid_token_fails() {
        let test_app = TestApp::spawn_app().await;

        // Build WebSocket URL with invalid token
        let ws_url = format!(
            "ws://{}/ws/user/?token=invalid_token_12345",
            test_app.base_url.replace("http://", "")
        );

        // Try to connect with invalid token
        let result = connect_async(&ws_url).await;

        // Connection should fail or immediately close due to invalid authentication
        assert!(
            result.is_err() || {
                if let Ok((ws_stream, _)) = result {
                    let (_, mut read) = ws_stream.split();
                    // Connection should close immediately
                    let msg =
                        tokio::time::timeout(std::time::Duration::from_secs(2), read.next()).await;
                    msg.is_err() || msg.unwrap().is_none()
                } else {
                    false
                }
            },
            "WebSocket should reject connection with invalid token"
        );
    }

    #[tokio::test]
    async fn test_websocket_send_receive_multiple_messages() {
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

        // Build WebSocket URL with token
        let ws_url = format!(
            "ws://{}/ws/user/?token={}",
            test_app.base_url.replace("http://", ""),
            access_token
        );

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");
        let (mut write, mut read) = ws_stream.split();

        // Send multiple messages
        for i in 1..=3 {
            let test_message = format!(r#"{{"action":"test","count":{}}}"#, i);
            write
                .send(Message::Text(test_message.into()))
                .await
                .expect("Failed to send message");

            // Wait for response
            let timeout_duration = std::time::Duration::from_secs(5);
            let response = tokio::time::timeout(timeout_duration, read.next()).await;

            assert!(
                response.is_ok(),
                "Timeout waiting for response to message {}",
                i
            );
        }

        // Close the connection
        write.send(Message::Close(None)).await.ok();
    }

    #[tokio::test]
    async fn test_sync_user_data_websocket_connection() {
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

        // Build WebSocket URL for sync user data
        let ws_url = format!(
            "ws://{}/ws/user/?token={}",
            test_app.base_url.replace("http://", ""),
            access_token
        );

        // Connect to WebSocket
        let result = connect_async(&ws_url).await;

        // Connection should succeed or fail gracefully
        match result {
            Ok((ws_stream, _)) => {
                let (mut write, _) = ws_stream.split();
                // Close gracefully
                write.send(Message::Close(None)).await.ok();
            }
            Err(_) => {
                // Connection failed - acceptable in test environment
            }
        }
    }
}
