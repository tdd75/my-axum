#[cfg(test)]
mod password_reset_repository_tests {
    use chrono::{Duration, Utc};
    use my_axum::core::context::Context;
    use my_axum::core::db::entity::password_reset_token;
    use my_axum::user::dto::user_dto::UserCreateDTO;
    use my_axum::user::repository::password_reset_repository;
    use my_axum::user::use_case::user::create_user_use_case;
    use sea_orm::{ActiveValue::Set, DbErr, TransactionTrait};
    use uuid::Uuid;

    use crate::setup::app::TestApp;

    async fn create_test_user(context: &Context<'_>) -> i32 {
        let dto = UserCreateDTO {
            email: format!("test{}@example.com", Uuid::new_v4()),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };
        let user = create_user_use_case::execute(context, dto)
            .await
            .unwrap()
            .data;
        user.id
    }

    #[tokio::test]
    async fn test_create_password_reset_token() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let user_id = create_test_user(&context).await;

                    let reset_token = password_reset_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("test_token_123".to_string()),
                        expires_at: Set(Utc::now().naive_utc() + Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    let created = password_reset_repository::create(&context, reset_token)
                        .await
                        .unwrap();

                    assert!(created.id > 0);
                    assert_eq!(created.user_id, user_id);
                    assert_eq!(created.token, "test_token_123");
                    assert_eq!(created.retry_count, 0);
                    assert!(created.created_at.is_some());

                    Ok(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_find_by_token_success() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let user_id = create_test_user(&context).await;

                    let reset_token = password_reset_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("find_me_token".to_string()),
                        expires_at: Set(Utc::now().naive_utc() + Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    password_reset_repository::create(&context, reset_token)
                        .await
                        .unwrap();

                    let found = password_reset_repository::find_by_token(&context, "find_me_token")
                        .await
                        .unwrap();

                    assert!(found.is_some());
                    let token = found.unwrap();
                    assert_eq!(token.token, "find_me_token");
                    assert_eq!(token.user_id, user_id);

                    Ok(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_find_by_token_not_found() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let found =
                        password_reset_repository::find_by_token(&context, "non_existent_token")
                            .await
                            .unwrap();

                    assert!(found.is_none());

                    Ok(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_update_password_reset_token() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let user_id = create_test_user(&context).await;

                    let reset_token = password_reset_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("update_token".to_string()),
                        expires_at: Set(Utc::now().naive_utc() + Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    let created = password_reset_repository::create(&context, reset_token)
                        .await
                        .unwrap();

                    // Update retry count
                    let mut active_model: password_reset_token::ActiveModel = created.into();
                    active_model.retry_count = Set(3);

                    let updated = password_reset_repository::update(&context, active_model)
                        .await
                        .unwrap();

                    assert_eq!(updated.retry_count, 3);

                    Ok(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_password_reset_token() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let user_id = create_test_user(&context).await;

                    let reset_token = password_reset_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("delete_token".to_string()),
                        expires_at: Set(Utc::now().naive_utc() + Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    let created = password_reset_repository::create(&context, reset_token)
                        .await
                        .unwrap();

                    // Delete the token
                    let active_model: password_reset_token::ActiveModel = created.into();
                    password_reset_repository::delete(&context, active_model)
                        .await
                        .unwrap();

                    // Verify it's deleted
                    let found = password_reset_repository::find_by_token(&context, "delete_token")
                        .await
                        .unwrap();
                    assert!(found.is_none());

                    Ok(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_expired_tokens() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let user_id = create_test_user(&context).await;

                    // Create expired token
                    let expired_token = password_reset_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("expired_token".to_string()),
                        expires_at: Set(Utc::now().naive_utc() - Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    password_reset_repository::create(&context, expired_token)
                        .await
                        .unwrap();

                    // Create valid token
                    let valid_token = password_reset_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("valid_token".to_string()),
                        expires_at: Set(Utc::now().naive_utc() + Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    password_reset_repository::create(&context, valid_token)
                        .await
                        .unwrap();

                    // Delete expired tokens
                    password_reset_repository::delete_expired(&context)
                        .await
                        .unwrap();

                    // Verify expired token is deleted
                    let found_expired =
                        password_reset_repository::find_by_token(&context, "expired_token")
                            .await
                            .unwrap();
                    assert!(found_expired.is_none());

                    // Verify valid token still exists
                    let found_valid =
                        password_reset_repository::find_by_token(&context, "valid_token")
                            .await
                            .unwrap();
                    assert!(found_valid.is_some());

                    Ok(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_by_user_id() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let user_id = create_test_user(&context).await;

                    // Create multiple tokens for the same user
                    let token1 = password_reset_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("token1".to_string()),
                        expires_at: Set(Utc::now().naive_utc() + Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    let token2 = password_reset_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("token2".to_string()),
                        expires_at: Set(Utc::now().naive_utc() + Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    password_reset_repository::create(&context, token1)
                        .await
                        .unwrap();
                    password_reset_repository::create(&context, token2)
                        .await
                        .unwrap();

                    // Delete all tokens for the user
                    password_reset_repository::delete_by_user_id(&context, user_id)
                        .await
                        .unwrap();

                    // Verify both tokens are deleted
                    let found1 = password_reset_repository::find_by_token(&context, "token1")
                        .await
                        .unwrap();
                    assert!(found1.is_none());

                    let found2 = password_reset_repository::find_by_token(&context, "token2")
                        .await
                        .unwrap();
                    assert!(found2.is_none());

                    Ok(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_by_user_id_different_users() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let user1_id = create_test_user(&context).await;
                    let user2_id = create_test_user(&context).await;

                    // Create tokens for both users
                    let token1 = password_reset_token::ActiveModel {
                        user_id: Set(user1_id),
                        token: Set("user1_token".to_string()),
                        expires_at: Set(Utc::now().naive_utc() + Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    let token2 = password_reset_token::ActiveModel {
                        user_id: Set(user2_id),
                        token: Set("user2_token".to_string()),
                        expires_at: Set(Utc::now().naive_utc() + Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    password_reset_repository::create(&context, token1)
                        .await
                        .unwrap();
                    password_reset_repository::create(&context, token2)
                        .await
                        .unwrap();

                    // Delete tokens for user1 only
                    password_reset_repository::delete_by_user_id(&context, user1_id)
                        .await
                        .unwrap();

                    // Verify user1 token is deleted
                    let found1 = password_reset_repository::find_by_token(&context, "user1_token")
                        .await
                        .unwrap();
                    assert!(found1.is_none());

                    // Verify user2 token still exists
                    let found2 = password_reset_repository::find_by_token(&context, "user2_token")
                        .await
                        .unwrap();
                    assert!(found2.is_some());

                    Ok(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_sets_created_at() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let user_id = create_test_user(&context).await;

                    let reset_token = password_reset_token::ActiveModel {
                        user_id: Set(user_id),
                        token: Set("created_at_test".to_string()),
                        expires_at: Set(Utc::now().naive_utc() + Duration::hours(1)),
                        retry_count: Set(0),
                        ..Default::default()
                    };

                    let created = password_reset_repository::create(&context, reset_token)
                        .await
                        .unwrap();

                    assert!(created.created_at.is_some());
                    let created_at = created.created_at.unwrap();

                    // Verify created_at is recent (within last minute)
                    let now = Utc::now().naive_utc();
                    let diff = now - created_at;
                    assert!(diff.num_seconds() < 60);

                    Ok(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_expired_with_no_expired_tokens() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    // Delete expired tokens when none exist - should not error
                    password_reset_repository::delete_expired(&context)
                        .await
                        .unwrap();

                    Ok(())
                })
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_by_user_id_with_no_tokens() {
        let test_app = TestApp::spawn_app().await;

        test_app
            .db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    let user_id = create_test_user(&context).await;

                    // Delete tokens for user with no tokens - should not error
                    password_reset_repository::delete_by_user_id(&context, user_id)
                        .await
                        .unwrap();

                    Ok(())
                })
            })
            .await
            .unwrap();
    }
}
