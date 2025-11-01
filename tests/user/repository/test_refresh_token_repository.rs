use crate::setup::app::TestApp;

use chrono::{Duration, Utc};
use my_axum::core::context::Context;
use my_axum::core::db::entity::{refresh_token, user};
use my_axum::user::repository::{
    refresh_token_repository::{self, RefreshTokenSearchParams},
    user_repository,
};
use sea_orm::{DbErr, Set};

#[tokio::test]
async fn test_create_refresh_token() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Create a user first
    let user_model = user::ActiveModel {
        email: Set("refresh_test@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Refresh".to_string())),
        last_name: Set(Some("Test".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let created_user = user_repository::create(&context, user_model).await?;

    // Create refresh token
    let expires_at = Utc::now().naive_utc() + Duration::hours(24);
    let refresh_token_model = refresh_token::ActiveModel {
        user_id: Set(created_user.id),
        token: Set("test_refresh_token_12345".to_string()),
        device_info: Set(Some("Test Device".to_string())),
        ip_address: Set(Some("192.168.1.1".to_string())),
        expires_at: Set(expires_at),
        created_at: Set(Some(chrono::Utc::now().naive_utc())),
        ..Default::default()
    };

    let created_token = refresh_token_repository::create(&context, refresh_token_model).await?;

    assert!(created_token.id > 0);
    assert_eq!(created_token.user_id, created_user.id);
    assert_eq!(created_token.token, "test_refresh_token_12345");
    assert_eq!(created_token.device_info, Some("Test Device".to_string()));
    assert_eq!(created_token.ip_address, Some("192.168.1.1".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_find_by_token_success() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Create user and refresh token
    let user_model = user::ActiveModel {
        email: Set("find_token@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Find".to_string())),
        last_name: Set(Some("Token".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let created_user = user_repository::create(&context, user_model).await?;

    let token_value = "find_token_test_12345";
    let refresh_token_model = refresh_token::ActiveModel {
        user_id: Set(created_user.id),
        token: Set(token_value.to_string()),
        device_info: Set(Some("Test Device".to_string())),
        ip_address: Set(Some("192.168.1.1".to_string())),
        expires_at: Set(Utc::now().naive_utc() + Duration::hours(24)),
        created_at: Set(Some(chrono::Utc::now().naive_utc())),
        ..Default::default()
    };
    refresh_token_repository::create(&context, refresh_token_model).await?;

    // Test find_by_token
    let result = refresh_token_repository::find_by_token(&context, token_value).await?;

    assert!(result.is_some());
    let found_token = result.unwrap();
    assert_eq!(found_token.token, token_value);
    assert_eq!(found_token.user_id, created_user.id);

    Ok(())
}

#[tokio::test]
async fn test_find_by_token_not_found() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    let result = refresh_token_repository::find_by_token(&context, "nonexistent_token").await?;
    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_find_by_user_and_token_success() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Create user and refresh token
    let user_model = user::ActiveModel {
        email: Set("user_token@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("User".to_string())),
        last_name: Set(Some("Token".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let created_user = user_repository::create(&context, user_model).await?;

    let token_value = "user_token_test_12345";
    let refresh_token_model = refresh_token::ActiveModel {
        user_id: Set(created_user.id),
        token: Set(token_value.to_string()),
        device_info: Set(Some("Test Device".to_string())),
        ip_address: Set(Some("192.168.1.1".to_string())),
        expires_at: Set(Utc::now().naive_utc() + Duration::hours(24)),
        created_at: Set(Some(chrono::Utc::now().naive_utc())),
        ..Default::default()
    };
    refresh_token_repository::create(&context, refresh_token_model).await?;

    // Test find_by_user_and_token
    let result =
        refresh_token_repository::find_by_user_and_token(&context, created_user.id, token_value)
            .await?;

    assert!(result.is_some());
    let found_token = result.unwrap();
    assert_eq!(found_token.token, token_value);
    assert_eq!(found_token.user_id, created_user.id);

    Ok(())
}

#[tokio::test]
async fn test_find_by_user_and_token_expired() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Create user and expired refresh token
    let user_model = user::ActiveModel {
        email: Set("expired_token@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Expired".to_string())),
        last_name: Set(Some("Token".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let created_user = user_repository::create(&context, user_model).await?;

    let token_value = "expired_token_test_12345";
    let refresh_token_model = refresh_token::ActiveModel {
        user_id: Set(created_user.id),
        token: Set(token_value.to_string()),
        device_info: Set(Some("Test Device".to_string())),
        ip_address: Set(Some("192.168.1.1".to_string())),
        expires_at: Set(Utc::now().naive_utc() - Duration::hours(1)), // Expired 1 hour ago
        created_at: Set(Some(chrono::Utc::now().naive_utc())),
        ..Default::default()
    };
    refresh_token_repository::create(&context, refresh_token_model).await?;

    // Test find_by_user_and_token with expired token
    let result =
        refresh_token_repository::find_by_user_and_token(&context, created_user.id, token_value)
            .await?;

    // Should return None because token is expired
    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_search_by_user_id() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Create user and multiple refresh tokens
    let user_model = user::ActiveModel {
        email: Set("search_tokens@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Search".to_string())),
        last_name: Set(Some("Tokens".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let created_user = user_repository::create(&context, user_model).await?;

    // Create multiple tokens for the user
    for i in 1..=3 {
        let refresh_token_model = refresh_token::ActiveModel {
            user_id: Set(created_user.id),
            token: Set(format!("search_token_{}", i)),
            device_info: Set(Some(format!("Device {}", i))),
            ip_address: Set(Some("192.168.1.1".to_string())),
            expires_at: Set(Utc::now().naive_utc() + Duration::hours(24)),
            created_at: Set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        refresh_token_repository::create(&context, refresh_token_model).await?;
    }

    // Search by user_id
    let params = RefreshTokenSearchParams {
        user_id: Some(created_user.id),
        ..Default::default()
    };
    let results = refresh_token_repository::search(&context, &params).await?;

    assert_eq!(results.len(), 3);
    for token in &results {
        assert_eq!(token.user_id, created_user.id);
    }

    Ok(())
}

#[tokio::test]
async fn test_search_expired_tokens() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Create user
    let user_model = user::ActiveModel {
        email: Set("expired_search@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Expired".to_string())),
        last_name: Set(Some("Search".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let created_user = user_repository::create(&context, user_model).await?;

    // Create expired tokens
    for i in 1..=2 {
        let refresh_token_model = refresh_token::ActiveModel {
            user_id: Set(created_user.id),
            token: Set(format!("expired_search_token_{}", i)),
            device_info: Set(Some(format!("Device {}", i))),
            ip_address: Set(Some("192.168.1.1".to_string())),
            expires_at: Set(Utc::now().naive_utc() - Duration::hours(i as i64)), // Expired
            ..Default::default()
        };
        refresh_token_repository::create(&context, refresh_token_model).await?;
    }

    // Create valid token
    let valid_token_model = refresh_token::ActiveModel {
        user_id: Set(created_user.id),
        token: Set("valid_search_token".to_string()),
        device_info: Set(Some("Valid Device".to_string())),
        ip_address: Set(Some("192.168.1.1".to_string())),
        expires_at: Set(Utc::now().naive_utc() + Duration::hours(24)), // Valid
        ..Default::default()
    };
    refresh_token_repository::create(&context, valid_token_model).await?;

    // Search for expired tokens
    let params = RefreshTokenSearchParams {
        user_id: Some(created_user.id),
        is_expired: Some(true),
        ..Default::default()
    };
    let expired_results = refresh_token_repository::search(&context, &params).await?;
    assert_eq!(expired_results.len(), 2);

    // Search for valid tokens
    let params = RefreshTokenSearchParams {
        user_id: Some(created_user.id),
        is_expired: Some(false),
        ..Default::default()
    };
    let valid_results = refresh_token_repository::search(&context, &params).await?;
    assert_eq!(valid_results.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_delete_by_token() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Create user and refresh token
    let user_model = user::ActiveModel {
        email: Set("delete_token@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Delete".to_string())),
        last_name: Set(Some("Token".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let created_user = user_repository::create(&context, user_model).await?;

    let token_value = "delete_token_test_12345";
    let refresh_token_model = refresh_token::ActiveModel {
        user_id: Set(created_user.id),
        token: Set(token_value.to_string()),
        device_info: Set(Some("Test Device".to_string())),
        ip_address: Set(Some("192.168.1.1".to_string())),
        expires_at: Set(Utc::now().naive_utc() + Duration::hours(24)),
        ..Default::default()
    };
    refresh_token_repository::create(&context, refresh_token_model).await?;

    // Verify token exists
    let before_delete = refresh_token_repository::find_by_token(&context, token_value).await?;
    assert!(before_delete.is_some());

    // Delete token
    refresh_token_repository::delete_by_token(&context, token_value).await?;

    // Verify token is deleted
    let after_delete = refresh_token_repository::find_by_token(&context, token_value).await?;
    assert!(after_delete.is_none());

    Ok(())
}

#[tokio::test]
async fn test_delete_by_tokens_batch() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Create user and multiple refresh tokens
    let user_model = user::ActiveModel {
        email: Set("batch_delete@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Batch".to_string())),
        last_name: Set(Some("Delete".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let created_user = user_repository::create(&context, user_model).await?;

    let mut token_values = Vec::new();
    for i in 1..=5 {
        let token_value = format!("batch_delete_token_{}", i);
        token_values.push(token_value.clone());

        let refresh_token_model = refresh_token::ActiveModel {
            user_id: Set(created_user.id),
            token: Set(token_value),
            device_info: Set(Some(format!("Device {}", i))),
            ip_address: Set(Some("192.168.1.1".to_string())),
            expires_at: Set(Utc::now().naive_utc() + Duration::hours(24)),
            ..Default::default()
        };
        refresh_token_repository::create(&context, refresh_token_model).await?;
    }

    // Verify all tokens exist
    let params = RefreshTokenSearchParams {
        user_id: Some(created_user.id),
        ..Default::default()
    };
    let before_delete = refresh_token_repository::search(&context, &params).await?;
    assert_eq!(before_delete.len(), 5);

    // Delete tokens in batch
    refresh_token_repository::delete_by_tokens(&context, &token_values).await?;

    // Verify all tokens are deleted
    let after_delete = refresh_token_repository::search(&context, &params).await?;
    assert_eq!(after_delete.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_search_with_pagination() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Create user and multiple refresh tokens
    let user_model = user::ActiveModel {
        email: Set("pagination_test@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Pagination".to_string())),
        last_name: Set(Some("Test".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let created_user = user_repository::create(&context, user_model).await?;

    // Create 10 tokens
    for i in 1..=10 {
        let refresh_token_model = refresh_token::ActiveModel {
            user_id: Set(created_user.id),
            token: Set(format!("pagination_token_{:02}", i)),
            device_info: Set(Some(format!("Device {}", i))),
            ip_address: Set(Some("192.168.1.1".to_string())),
            expires_at: Set(Utc::now().naive_utc() + Duration::hours(24)),
            ..Default::default()
        };
        refresh_token_repository::create(&context, refresh_token_model).await?;
    }

    // Test pagination - first page
    let params = RefreshTokenSearchParams {
        user_id: Some(created_user.id),
        page: Some(1),
        page_size: Some(5),
        ..Default::default()
    };
    let page1 = refresh_token_repository::search(&context, &params).await?;
    assert_eq!(page1.len(), 5);

    // Test pagination - second page
    let params = RefreshTokenSearchParams {
        user_id: Some(created_user.id),
        page: Some(2),
        page_size: Some(5),
        ..Default::default()
    };
    let page2 = refresh_token_repository::search(&context, &params).await?;
    assert_eq!(page2.len(), 5);

    // Verify different results
    let page1_tokens: Vec<String> = page1.iter().map(|t| t.token.clone()).collect();
    let page2_tokens: Vec<String> = page2.iter().map(|t| t.token.clone()).collect();

    // Pages should have different tokens
    for token in &page1_tokens {
        assert!(!page2_tokens.contains(token));
    }

    Ok(())
}
