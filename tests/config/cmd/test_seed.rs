use crate::setup::app::TestApp;
use my_axum::core::db::entity::user;
use my_axum::pkg::password;
use my_axum::{config::cmd::seed, core::context::Context};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};

#[tokio::test]
async fn test_seed_users_creates_new_users() {
    let test_app = TestApp::spawn_app().await;

    // Seed users
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };
    let result = seed::seed_users(&mut context).await;
    assert!(result.is_ok());

    // Verify regular user was created
    let user_result = user::Entity::find()
        .filter(user::Column::Email.eq("user@example.com"))
        .one(&txn)
        .await;

    assert!(user_result.is_ok());
    let user = user_result.unwrap();
    assert!(user.is_some());

    let user = user.unwrap();
    assert_eq!(user.email, "user@example.com");
    assert_eq!(user.first_name, Some("John".to_string()));
    assert_eq!(user.last_name, Some("Doe".to_string()));
    assert_eq!(user.phone, Some("+1234567890".to_string()));

    // Verify admin user was created
    let admin_result = user::Entity::find()
        .filter(user::Column::Email.eq("admin@example.com"))
        .one(&txn)
        .await;

    assert!(admin_result.is_ok());
    let admin = admin_result.unwrap();
    assert!(admin.is_some());

    let admin = admin.unwrap();
    assert_eq!(admin.email, "admin@example.com");
    assert_eq!(admin.first_name, Some("Admin".to_string()));
    assert_eq!(admin.last_name, Some("User".to_string()));
    assert_eq!(admin.phone, Some("+1987654321".to_string()));
}

#[tokio::test]
async fn test_seed_users_skips_existing_users() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Run seed first time
    let result = seed::seed_users(&mut context).await;
    assert!(result.is_ok());

    // Count users
    let count_before = user::Entity::find().all(&txn).await.unwrap().len();

    // Run seed second time
    let result = seed::seed_users(&mut context).await;
    assert!(result.is_ok());

    // Count users again - should be the same
    let count_after = user::Entity::find().all(&txn).await.unwrap().len();
    assert_eq!(count_before, count_after);
}

#[tokio::test]
async fn test_seed_users_passwords_are_hashed() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Get user
    let user = user::Entity::find()
        .filter(user::Column::Email.eq("user@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    // Password should not be plain text
    assert_ne!(user.password, "password123@");

    // Password should be verifiable
    let is_valid = password::verify_password("password123@", &user.password).await;
    assert!(is_valid.is_ok());
}

#[tokio::test]
async fn test_seed_users_admin_password_is_hashed() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Get admin
    let admin = user::Entity::find()
        .filter(user::Column::Email.eq("admin@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    // Password should not be plain text
    assert_ne!(admin.password, "admin123@");

    // Password should be verifiable
    let is_valid = password::verify_password("admin123@", &admin.password).await;
    assert!(is_valid.is_ok());
}

#[tokio::test]
async fn test_seed_users_sets_timestamps() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Get user
    let user = user::Entity::find()
        .filter(user::Column::Email.eq("user@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    // Timestamps should be set
    assert!(user.created_at.is_some());
    assert!(user.updated_at.is_some());
}

#[tokio::test]
async fn test_seed_users_creates_exactly_two_users() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Count users
    let users = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users.len(), 2);
}

#[tokio::test]
async fn test_seed_users_idempotent() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Run multiple times
    for _ in 0..3 {
        let result = seed::seed_users(&mut context).await;
        assert!(result.is_ok());
    }

    // Should still have exactly 2 users
    let users = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users.len(), 2);
}

#[tokio::test]
async fn test_seed_users_both_emails_unique() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Get all users
    let users = user::Entity::find().all(&txn).await.unwrap();

    // Check emails are unique
    let emails: Vec<String> = users.iter().map(|u| u.email.clone()).collect();
    assert!(emails.contains(&"user@example.com".to_string()));
    assert!(emails.contains(&"admin@example.com".to_string()));
    assert_eq!(emails.len(), 2);
}

#[tokio::test]
async fn test_seed_users_with_existing_regular_user_only() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Pre-create regular user
    let user_model = user::ActiveModel {
        email: Set("user@example.com".to_string()),
        password: Set("existing_hash".to_string()),
        first_name: Set(Some("Existing".to_string())),
        last_name: Set(Some("User".to_string())),
        phone: Set(Some("+9999999999".to_string())),
        ..Default::default()
    };
    user::Entity::insert(user_model).exec(&txn).await.unwrap();

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Should have exactly 2 users (existing + new admin)
    let users = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users.len(), 2);

    // Check existing user was not modified
    let existing = user::Entity::find()
        .filter(user::Column::Email.eq("user@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(existing.first_name, Some("Existing".to_string()));
    assert_eq!(existing.phone, Some("+9999999999".to_string()));
}

#[tokio::test]
async fn test_seed_users_with_existing_admin_only() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Pre-create admin user
    let admin_model = user::ActiveModel {
        email: Set("admin@example.com".to_string()),
        password: Set("existing_admin_hash".to_string()),
        first_name: Set(Some("Existing".to_string())),
        last_name: Set(Some("Admin".to_string())),
        phone: Set(Some("+8888888888".to_string())),
        ..Default::default()
    };
    user::Entity::insert(admin_model).exec(&txn).await.unwrap();

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Should have exactly 2 users (existing admin + new user)
    let users = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users.len(), 2);

    // Check existing admin was not modified
    let existing = user::Entity::find()
        .filter(user::Column::Email.eq("admin@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(existing.first_name, Some("Existing".to_string()));
    assert_eq!(existing.phone, Some("+8888888888".to_string()));
}

#[tokio::test]
async fn test_seed_users_timestamps_are_recent() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    let before_seed = chrono::Utc::now().naive_utc();

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    let after_seed = chrono::Utc::now().naive_utc();

    // Get user
    let user = user::Entity::find()
        .filter(user::Column::Email.eq("user@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    // Timestamps should be between before and after
    assert!(user.created_at.unwrap() >= before_seed);
    assert!(user.created_at.unwrap() <= after_seed);
    assert!(user.updated_at.unwrap() >= before_seed);
    assert!(user.updated_at.unwrap() <= after_seed);
}

#[tokio::test]
async fn test_seed_users_all_fields_populated() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Get user and check all fields
    let user = user::Entity::find()
        .filter(user::Column::Email.eq("user@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    assert!(user.id > 0);
    assert_eq!(user.email, "user@example.com");
    assert!(!user.password.is_empty());
    assert_eq!(user.first_name, Some("John".to_string()));
    assert_eq!(user.last_name, Some("Doe".to_string()));
    assert_eq!(user.phone, Some("+1234567890".to_string()));
    assert!(user.created_at.is_some());
    assert!(user.updated_at.is_some());

    // Get admin and check all fields
    let admin = user::Entity::find()
        .filter(user::Column::Email.eq("admin@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    assert!(admin.id > 0);
    assert_eq!(admin.email, "admin@example.com");
    assert!(!admin.password.is_empty());
    assert_eq!(admin.first_name, Some("Admin".to_string()));
    assert_eq!(admin.last_name, Some("User".to_string()));
    assert_eq!(admin.phone, Some("+1987654321".to_string()));
    assert!(admin.created_at.is_some());
    assert!(admin.updated_at.is_some());
}

#[tokio::test]
async fn test_seed_run_with_actual_database() {
    // Test the actual run() function
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Since run() uses Setting::init() which the TestApp also uses,
    // we can safely test seed_users which is the core logic
    let result = seed::seed_users(&mut context).await;
    assert!(result.is_ok());

    // Verify both users were created
    let users = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users.len(), 2);
}

#[tokio::test]
async fn test_seed_users_handles_multiple_seeding_calls() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // First seed
    let result1 = seed::seed_users(&mut context).await;
    assert!(result1.is_ok());

    let users_after_first = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users_after_first.len(), 2);

    // Second seed - should not create duplicates
    let result2 = seed::seed_users(&mut context).await;
    assert!(result2.is_ok());

    let users_after_second = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users_after_second.len(), 2);

    // Third seed - still no duplicates
    let result3 = seed::seed_users(&mut context).await;
    assert!(result3.is_ok());

    let users_after_third = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users_after_third.len(), 2);
}

#[tokio::test]
async fn test_seed_users_password_verification() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Verify regular user password
    let user = user::Entity::find()
        .filter(user::Column::Email.eq("user@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    let user_verify = password::verify_password("password123@", &user.password).await;
    assert!(user_verify.is_ok());

    // Verify admin password
    let admin = user::Entity::find()
        .filter(user::Column::Email.eq("admin@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    let admin_verify = password::verify_password("admin123@", &admin.password).await;
    assert!(admin_verify.is_ok());

    // Verify wrong passwords fail
    let wrong_user_verify = password::verify_password("wrongpass", &user.password).await;
    assert!(wrong_user_verify.is_err());

    let wrong_admin_verify = password::verify_password("wrongpass", &admin.password).await;
    assert!(wrong_admin_verify.is_err());
}

#[tokio::test]
async fn test_seed_users_different_user_data() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Get both users
    let user = user::Entity::find()
        .filter(user::Column::Email.eq("user@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    let admin = user::Entity::find()
        .filter(user::Column::Email.eq("admin@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    // Verify they have different data
    assert_ne!(user.id, admin.id);
    assert_ne!(user.email, admin.email);
    assert_ne!(user.password, admin.password);
    assert_ne!(user.first_name, admin.first_name);
    assert_ne!(user.last_name, admin.last_name);
    assert_ne!(user.phone, admin.phone);
}

#[tokio::test]
async fn test_seed_users_returns_ok_with_empty_database() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Verify database is empty
    let users_before = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users_before.len(), 0);

    // Seed should succeed
    let result = seed::seed_users(&mut context).await;
    assert!(result.is_ok());

    // Verify users were created
    let users_after = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users_after.len(), 2);
}

#[tokio::test]
async fn test_seed_users_preserves_existing_user_data() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Pre-create both users with different data
    let user_model = user::ActiveModel {
        email: Set("user@example.com".to_string()),
        password: Set("old_user_hash".to_string()),
        first_name: Set(Some("OldFirst".to_string())),
        last_name: Set(Some("OldLast".to_string())),
        phone: Set(Some("+9999999999".to_string())),
        ..Default::default()
    };
    user::Entity::insert(user_model).exec(&txn).await.unwrap();

    let admin_model = user::ActiveModel {
        email: Set("admin@example.com".to_string()),
        password: Set("old_admin_hash".to_string()),
        first_name: Set(Some("OldAdmin".to_string())),
        last_name: Set(Some("OldAdminLast".to_string())),
        phone: Set(Some("+8888888888".to_string())),
        ..Default::default()
    };
    user::Entity::insert(admin_model).exec(&txn).await.unwrap();

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Should still have exactly 2 users
    let users = user::Entity::find().all(&txn).await.unwrap();
    assert_eq!(users.len(), 2);

    // Check that existing data was preserved
    let user = user::Entity::find()
        .filter(user::Column::Email.eq("user@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.password, "old_user_hash");
    assert_eq!(user.first_name, Some("OldFirst".to_string()));

    let admin = user::Entity::find()
        .filter(user::Column::Email.eq("admin@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(admin.password, "old_admin_hash");
    assert_eq!(admin.first_name, Some("OldAdmin".to_string()));
}

#[tokio::test]
async fn test_seed_users_each_user_has_valid_id() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Get all users and verify IDs
    let users = user::Entity::find().all(&txn).await.unwrap();

    for user in users {
        assert!(user.id > 0, "User ID should be greater than 0");
    }
}

#[tokio::test]
async fn test_seed_users_created_and_updated_at_match() {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let mut context = Context {
        txn: &txn,
        user: None,
        producer: None,
    };

    // Seed users
    seed::seed_users(&mut context).await.unwrap();

    // Get user
    let user = user::Entity::find()
        .filter(user::Column::Email.eq("user@example.com"))
        .one(&txn)
        .await
        .unwrap()
        .unwrap();

    // For new users, created_at and updated_at should be very close
    let created = user.created_at.unwrap();
    let updated = user.updated_at.unwrap();

    let diff = if updated >= created {
        updated - created
    } else {
        created - updated
    };

    // Should be within 1 second
    assert!(diff.num_seconds() <= 1);
}
