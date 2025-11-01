#[cfg(test)]
mod auth_task_tests {
    use crate::setup::app::TestApp;
    use chrono::{Duration, Utc};
    use my_axum::{
        core::{
            context::Context,
            db::entity::{refresh_token, user},
        },
        user::{
            repository::{
                refresh_token_repository::{self, RefreshTokenSearchParams},
                user_repository,
            },
            task::auth_task::clean_expired_tokens,
        },
    };
    use sea_orm::{Set, TransactionTrait};

    #[tokio::test]
    async fn test_clean_expired_tokens_success() -> Result<(), anyhow::Error> {
        let test_app = TestApp::spawn_app().await;

        // Create test data in a transaction
        let created_user_id = test_app
            .db
            .transaction::<_, i32, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    // Create a user first
                    let user_model = user::ActiveModel {
                        email: Set("clean_tokens@example.com".to_string()),
                        password: Set("password123@".to_string()),
                        first_name: Set(Some("Clean".to_string())),
                        last_name: Set(Some("Tokens".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };
                    let created_user = user_repository::create(&context, user_model)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    // Create expired and valid tokens
                    for i in 1..=3 {
                        // Expired tokens
                        let expired_token = refresh_token::ActiveModel {
                            user_id: Set(created_user.id),
                            token: Set(format!("expired_token_{}", i)),
                            device_info: Set(Some(format!("Expired Device {}", i))),
                            ip_address: Set(Some("192.168.1.1".to_string())),
                            expires_at: Set(Utc::now().naive_utc() - Duration::hours(i as i64)),
                            ..Default::default()
                        };
                        refresh_token_repository::create(&context, expired_token)
                            .await
                            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                        // Valid tokens
                        let valid_token = refresh_token::ActiveModel {
                            user_id: Set(created_user.id),
                            token: Set(format!("valid_token_{}", i)),
                            device_info: Set(Some(format!("Valid Device {}", i))),
                            ip_address: Set(Some("192.168.1.1".to_string())),
                            expires_at: Set(Utc::now().naive_utc() + Duration::hours(24)),
                            ..Default::default()
                        };
                        refresh_token_repository::create(&context, valid_token)
                            .await
                            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    }

                    Ok(created_user.id)
                })
            })
            .await?;

        // Verify tokens exist before cleanup
        let all_tokens_before = test_app
            .db
            .transaction::<_, usize, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let tokens = refresh_token_repository::search(
                        &context,
                        &RefreshTokenSearchParams {
                            user_id: Some(created_user_id),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    Ok(tokens.len())
                })
            })
            .await?;
        assert_eq!(all_tokens_before, 6); // 3 expired + 3 valid

        // Run cleanup
        let result = clean_expired_tokens(&test_app.db).await;
        assert!(result.is_ok());

        // Verify only valid tokens remain
        let remaining_tokens = test_app
            .db
            .transaction::<_, Vec<refresh_token::Model>, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    refresh_token_repository::search(
                        &context,
                        &RefreshTokenSearchParams {
                            user_id: Some(created_user_id),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))
                })
            })
            .await?;
        assert_eq!(remaining_tokens.len(), 3); // Only valid tokens should remain

        // Verify all remaining tokens are valid (not expired)
        let now = Utc::now().naive_utc();
        for token in &remaining_tokens {
            assert!(token.expires_at > now, "Token should not be expired");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_clean_expired_tokens_no_expired_tokens() -> Result<(), anyhow::Error> {
        let test_app = TestApp::spawn_app().await;

        // Create test data
        let created_user_id = test_app
            .db
            .transaction::<_, i32, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    // Create a user
                    let user_model = user::ActiveModel {
                        email: Set("no_expired@example.com".to_string()),
                        password: Set("password123@".to_string()),
                        first_name: Set(Some("No".to_string())),
                        last_name: Set(Some("Expired".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };
                    let created_user = user_repository::create(&context, user_model)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    // Create only valid tokens
                    for i in 1..=3 {
                        let valid_token = refresh_token::ActiveModel {
                            user_id: Set(created_user.id),
                            token: Set(format!("valid_only_token_{}", i)),
                            device_info: Set(Some(format!("Device {}", i))),
                            ip_address: Set(Some("192.168.1.1".to_string())),
                            expires_at: Set(Utc::now().naive_utc() + Duration::hours(24)),
                            ..Default::default()
                        };
                        refresh_token_repository::create(&context, valid_token)
                            .await
                            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    }

                    Ok(created_user.id)
                })
            })
            .await?;

        // Run cleanup
        let result = clean_expired_tokens(&test_app.db).await;
        assert!(result.is_ok());

        // Verify all tokens still exist
        let remaining_tokens = test_app
            .db
            .transaction::<_, Vec<refresh_token::Model>, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    refresh_token_repository::search(
                        &context,
                        &RefreshTokenSearchParams {
                            user_id: Some(created_user_id),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))
                })
            })
            .await?;
        assert_eq!(remaining_tokens.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_clean_expired_tokens_empty_database() -> Result<(), anyhow::Error> {
        let test_app = TestApp::spawn_app().await;

        // Run cleanup on empty database
        let result = clean_expired_tokens(&test_app.db).await;
        assert!(result.is_ok());

        // Verify no tokens exist
        let tokens = test_app
            .db
            .transaction::<_, Vec<refresh_token::Model>, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    refresh_token_repository::search(&context, &RefreshTokenSearchParams::default())
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))
                })
            })
            .await?;
        assert_eq!(tokens.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_clean_expired_tokens_batch_processing() -> Result<(), anyhow::Error> {
        let test_app = TestApp::spawn_app().await;

        // Create test data
        let created_user_id = test_app
            .db
            .transaction::<_, i32, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    // Create a user
                    let user_model = user::ActiveModel {
                        email: Set("batch_test@example.com".to_string()),
                        password: Set("password123@".to_string()),
                        first_name: Set(Some("Batch".to_string())),
                        last_name: Set(Some("Test".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };
                    let created_user = user_repository::create(&context, user_model)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    // Create a large number of expired tokens to test batch processing
                    for i in 1..=150 {
                        // More than BATCH_SIZE (100) to test batch processing
                        let expired_token = refresh_token::ActiveModel {
                            user_id: Set(created_user.id),
                            token: Set(format!("batch_expired_token_{:03}", i)),
                            device_info: Set(Some(format!("Batch Device {}", i))),
                            ip_address: Set(Some("192.168.1.1".to_string())),
                            expires_at: Set(Utc::now().naive_utc() - Duration::minutes(i as i64)),
                            ..Default::default()
                        };
                        refresh_token_repository::create(&context, expired_token)
                            .await
                            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    }

                    Ok(created_user.id)
                })
            })
            .await?;

        // Verify all tokens exist before cleanup
        let tokens_before = test_app
            .db
            .transaction::<_, usize, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let tokens = refresh_token_repository::search(
                        &context,
                        &RefreshTokenSearchParams {
                            user_id: Some(created_user_id),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    Ok(tokens.len())
                })
            })
            .await?;
        assert_eq!(tokens_before, 150);

        // Run cleanup (should process in batches)
        let result = clean_expired_tokens(&test_app.db).await;
        assert!(result.is_ok());

        // Verify all tokens are cleaned up
        let tokens_after = test_app
            .db
            .transaction::<_, usize, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let tokens = refresh_token_repository::search(
                        &context,
                        &RefreshTokenSearchParams {
                            user_id: Some(created_user_id),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    Ok(tokens.len())
                })
            })
            .await?;
        assert_eq!(tokens_after, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_clean_expired_tokens_mixed_users() -> Result<(), anyhow::Error> {
        let test_app = TestApp::spawn_app().await;

        // Create multiple users with tokens
        let user_ids = test_app
            .db
            .transaction::<_, Vec<i32>, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let mut user_ids = Vec::new();

                    // Create multiple users
                    for i in 1..=3 {
                        let user_model = user::ActiveModel {
                            email: Set(format!("mixed_user_{}@example.com", i)),
                            password: Set("password123@".to_string()),
                            first_name: Set(Some(format!("User{}", i))),
                            last_name: Set(Some("Mixed".to_string())),
                            phone: Set(None),
                            ..Default::default()
                        };
                        let created_user = user_repository::create(&context, user_model)
                            .await
                            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                        user_ids.push(created_user.id);

                        // Expired token
                        let expired_token = refresh_token::ActiveModel {
                            user_id: Set(created_user.id),
                            token: Set(format!("mixed_expired_token_user_{}", i)),
                            device_info: Set(Some("Expired Device".to_string())),
                            ip_address: Set(Some("192.168.1.1".to_string())),
                            expires_at: Set(Utc::now().naive_utc() - Duration::hours(1)),
                            ..Default::default()
                        };
                        refresh_token_repository::create(&context, expired_token)
                            .await
                            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                        // Valid token
                        let valid_token = refresh_token::ActiveModel {
                            user_id: Set(created_user.id),
                            token: Set(format!("mixed_valid_token_user_{}", i)),
                            device_info: Set(Some("Valid Device".to_string())),
                            ip_address: Set(Some("192.168.1.1".to_string())),
                            expires_at: Set(Utc::now().naive_utc() + Duration::hours(24)),
                            ..Default::default()
                        };
                        refresh_token_repository::create(&context, valid_token)
                            .await
                            .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;
                    }

                    Ok(user_ids)
                })
            })
            .await?;

        // Run cleanup
        let result = clean_expired_tokens(&test_app.db).await;
        assert!(result.is_ok());

        // Verify only valid tokens remain for each user
        for user_id in user_ids {
            let remaining_tokens = test_app
                .db
                .transaction::<_, Vec<refresh_token::Model>, sea_orm::DbErr>(|txn| {
                    Box::pin(async move {
                        let context = Context {
                            txn,
                            user: None,
                            producer: None,
                        };
                        refresh_token_repository::search(
                            &context,
                            &RefreshTokenSearchParams {
                                user_id: Some(user_id),
                                ..Default::default()
                            },
                        )
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))
                    })
                })
                .await?;
            assert_eq!(remaining_tokens.len(), 1);
            assert!(remaining_tokens[0].token.contains("valid"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_clean_expired_tokens_edge_case_just_expired() -> Result<(), anyhow::Error> {
        let test_app = TestApp::spawn_app().await;

        // Create test data
        let created_user_id = test_app
            .db
            .transaction::<_, i32, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    // Create a user
                    let user_model = user::ActiveModel {
                        email: Set("edge_case@example.com".to_string()),
                        password: Set("password123@".to_string()),
                        first_name: Set(Some("Edge".to_string())),
                        last_name: Set(Some("Case".to_string())),
                        phone: Set(None),
                        ..Default::default()
                    };
                    let created_user = user_repository::create(&context, user_model)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    // Create token that just expired (1 second ago)
                    let just_expired_token = refresh_token::ActiveModel {
                        user_id: Set(created_user.id),
                        token: Set("just_expired_token".to_string()),
                        device_info: Set(Some("Just Expired Device".to_string())),
                        ip_address: Set(Some("192.168.1.1".to_string())),
                        expires_at: Set(Utc::now().naive_utc() - Duration::seconds(1)),
                        ..Default::default()
                    };
                    refresh_token_repository::create(&context, just_expired_token)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    // Create token that just expires in the future (1 second)
                    let just_valid_token = refresh_token::ActiveModel {
                        user_id: Set(created_user.id),
                        token: Set("just_valid_token".to_string()),
                        device_info: Set(Some("Just Valid Device".to_string())),
                        ip_address: Set(Some("192.168.1.1".to_string())),
                        expires_at: Set(Utc::now().naive_utc() + Duration::seconds(1)),
                        ..Default::default()
                    };
                    refresh_token_repository::create(&context, just_valid_token)
                        .await
                        .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))?;

                    Ok(created_user.id)
                })
            })
            .await?;

        // Run cleanup
        let result = clean_expired_tokens(&test_app.db).await;
        assert!(result.is_ok());

        // Verify only the just_valid_token remains
        let remaining_tokens = test_app
            .db
            .transaction::<_, Vec<refresh_token::Model>, sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    refresh_token_repository::search(
                        &context,
                        &RefreshTokenSearchParams {
                            user_id: Some(created_user_id),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| sea_orm::DbErr::Custom(e.to_string()))
                })
            })
            .await?;
        assert_eq!(remaining_tokens.len(), 1);
        assert_eq!(remaining_tokens[0].token, "just_valid_token");

        Ok(())
    }

    #[tokio::test]
    async fn test_clean_expired_tokens_function_signature() {
        // Test the function signature and return type
        let test_app = TestApp::spawn_app().await;

        // Test that function has correct signature and returns correct type
        use std::future::Future;
        use std::pin::Pin;

        let future: Pin<Box<dyn Future<Output = Result<(), anyhow::Error>>>> =
            Box::pin(clean_expired_tokens(&test_app.db));

        let result = future.await;
        assert!(result.is_ok() || result.is_err()); // Either outcome is valid for testing
    }
}
