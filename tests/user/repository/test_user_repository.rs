use my_axum::core::context::Context;
use my_axum::core::db::entity::user;
use my_axum::core::db::ordering::{OrderByField, SortOrder};
use my_axum::user::repository::user_repository::{
    self, UserOrderBy, UserOrderByField, UserSearchParams,
};
use sea_orm::{DbErr, Set};
use std::sync::Arc;

use crate::setup::app::TestApp;

#[tokio::test]
async fn test_find_by_id_success() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create a user
    let user_model = user::ActiveModel {
        email: Set("find_by_id@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Find".to_string())),
        last_name: Set(Some("ById".to_string())),
        phone: Set(Some("123-456-7890".to_string())),
        ..Default::default()
    };

    let created_user = user_repository::create(&context, user_model).await?;

    // Test find_by_id
    let result = user_repository::find_by_id(&context, created_user.id).await?;

    assert!(result.is_some());
    let user = result.unwrap();
    assert_eq!(user.email, "find_by_id@example.com");

    Ok(())
}

#[tokio::test]
async fn test_find_by_id_not_found() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Test with non-existent ID
    let result = user_repository::find_by_id(&context, 999999).await?;

    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_find_by_email_success() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create a user
    let user_model = user::ActiveModel {
        email: Set("find_by_email@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Find".to_string())),
        last_name: Set(Some("ByEmail".to_string())),
        phone: Set(Some("123-456-7890".to_string())),
        ..Default::default()
    };

    user_repository::create(&context, user_model).await?;

    // Test find_by_email
    let result = user_repository::find_by_email(&context, "find_by_email@example.com").await?;

    assert!(result.is_some());
    let user = result.unwrap();
    assert_eq!(user.email, "find_by_email@example.com");

    Ok(())
}

#[tokio::test]
async fn test_find_by_email_not_found() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Test with non-existent email
    let result = user_repository::find_by_email(&context, "nonexistent@example.com").await?;

    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_create_user() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    let user_model = user::ActiveModel {
        email: Set("create_user@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Create".to_string())),
        last_name: Set(Some("User".to_string())),
        phone: Set(Some("123-456-7890".to_string())),
        ..Default::default()
    };

    let created_user = user_repository::create(&context, user_model).await?;

    assert!(created_user.id > 0);
    assert_eq!(created_user.email, "create_user@example.com");
    assert!(created_user.created_at.is_some());
    assert!(created_user.updated_at.is_some());

    Ok(())
}

#[tokio::test]
async fn test_delete_user() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create a user
    let user_model = user::ActiveModel {
        email: Set("delete_user@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Delete".to_string())),
        last_name: Set(Some("User".to_string())),
        phone: Set(Some("123-456-7890".to_string())),
        ..Default::default()
    };

    let created_user = user_repository::create(&context, user_model).await?;

    // Delete the user
    let active_model: user::ActiveModel = created_user.clone().into();
    user_repository::delete(&context, active_model).await?;

    // Verify deletion
    let result = user_repository::find_by_id(&context, created_user.id).await?;
    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_search_no_filters() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create multiple users
    let users = vec![
        user::ActiveModel {
            email: Set("search1@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Alice".to_string())),
            last_name: Set(Some("Johnson".to_string())),
            phone: Set(Some("123-456-7890".to_string())),
            ..Default::default()
        },
        user::ActiveModel {
            email: Set("search2@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Bob".to_string())),
            last_name: Set(Some("Smith".to_string())),
            phone: Set(Some("123-456-7891".to_string())),
            ..Default::default()
        },
        user::ActiveModel {
            email: Set("search3@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Charlie".to_string())),
            last_name: Set(Some("Brown".to_string())),
            phone: Set(Some("123-456-7892".to_string())),
            ..Default::default()
        },
    ];

    for user in users {
        user_repository::create(&context, user).await?;
    }

    // Search without filters (should return all users with default pagination)
    let result = user_repository::search(
        &context,
        &UserSearchParams {
            ..Default::default()
        },
    )
    .await?;

    assert!(result.len() >= 3); // Should contain at least the 3 users we created

    Ok(())
}

#[tokio::test]
async fn test_search_by_email() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create users with specific emails
    let users = vec![
        user::ActiveModel {
            email: Set("email_filter_test1@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Alice".to_string())),
            last_name: Set(Some("Johnson".to_string())),
            ..Default::default()
        },
        user::ActiveModel {
            email: Set("email_filter_test2@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Bob".to_string())),
            last_name: Set(Some("Smith".to_string())),
            ..Default::default()
        },
        user::ActiveModel {
            email: Set("other@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Charlie".to_string())),
            last_name: Set(Some("Brown".to_string())),
            ..Default::default()
        },
    ];

    for user in users {
        user_repository::create(&context, user).await?;
    }

    // Search by email containing "email_filter_test"
    let result = user_repository::search(
        &context,
        &UserSearchParams {
            email: Some("email_filter_test"),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|u| u.email.contains("email_filter_test")));

    Ok(())
}

#[tokio::test]
async fn test_search_by_first_name() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create users with specific first names
    let users = vec![
        user::ActiveModel {
            email: Set("first_name_test1@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Alexander".to_string())),
            last_name: Set(Some("Johnson".to_string())),
            ..Default::default()
        },
        user::ActiveModel {
            email: Set("first_name_test2@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Alexandra".to_string())),
            last_name: Set(Some("Smith".to_string())),
            ..Default::default()
        },
        user::ActiveModel {
            email: Set("first_name_test3@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Bob".to_string())),
            last_name: Set(Some("Brown".to_string())),
            ..Default::default()
        },
    ];

    for user in users {
        user_repository::create(&context, user).await?;
    }

    // Search by first name containing "Alex"
    let result = user_repository::search(
        &context,
        &UserSearchParams {
            first_name: Some("Alex"),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 2);
    assert!(
        result
            .iter()
            .all(|u| u.first_name.as_ref().unwrap().contains("Alex"))
    );

    Ok(())
}

#[tokio::test]
async fn test_search_by_last_name() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create users with specific last names
    let users = vec![
        user::ActiveModel {
            email: Set("last_name_test1@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Alice".to_string())),
            last_name: Set(Some("Johnson".to_string())),
            ..Default::default()
        },
        user::ActiveModel {
            email: Set("last_name_test2@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Bob".to_string())),
            last_name: Set(Some("Johnson".to_string())),
            ..Default::default()
        },
        user::ActiveModel {
            email: Set("last_name_test3@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Charlie".to_string())),
            last_name: Set(Some("Smith".to_string())),
            ..Default::default()
        },
    ];

    for user in users {
        user_repository::create(&context, user).await?;
    }

    // Search by last name "Johnson"
    let result = user_repository::search(
        &context,
        &UserSearchParams {
            last_name: Some("Johnson"),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 2);
    assert!(
        result
            .iter()
            .all(|u| u.last_name.as_ref().unwrap().contains("Johnson"))
    );

    Ok(())
}

#[tokio::test]
async fn test_search_with_multiple_filters() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create users for complex filtering
    let users = vec![
        user::ActiveModel {
            email: Set("multi_filter_alice@test.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Alice".to_string())),
            last_name: Set(Some("Johnson".to_string())),
            ..Default::default()
        },
        user::ActiveModel {
            email: Set("multi_filter_bob@test.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Alice".to_string())),
            last_name: Set(Some("Smith".to_string())),
            ..Default::default()
        },
        user::ActiveModel {
            email: Set("other_domain@example.com".to_string()),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Alice".to_string())),
            last_name: Set(Some("Johnson".to_string())),
            ..Default::default()
        },
    ];

    for user in users {
        user_repository::create(&context, user).await?;
    }

    // Search with multiple filters: email contains "test" AND first_name contains "Alice" AND last_name contains "Johnson"
    let result = user_repository::search(
        &context,
        &UserSearchParams {
            email: Some("test"),
            first_name: Some("Alice"),
            last_name: Some("Johnson"),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 1);
    let user = &result[0];
    assert!(user.email.contains("test"));
    assert_eq!(user.first_name.as_ref().unwrap(), "Alice");
    assert_eq!(user.last_name.as_ref().unwrap(), "Johnson");

    Ok(())
}

#[tokio::test]
async fn test_search_pagination() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create multiple users for pagination testing
    for i in 1..=15 {
        let user = user::ActiveModel {
            email: Set(format!("pagination_test_{}@example.com", i)),
            password: Set("password123@".to_string()),
            first_name: Set(Some(format!("User{}", i))),
            last_name: Set(Some("Test".to_string())),
            ..Default::default()
        };
        user_repository::create(&context, user).await?;
    }

    // Create ordering for consistent pagination
    let order_by = vec![UserOrderBy {
        field: UserOrderByField::Id,
        order: SortOrder::Asc,
    }];

    // Test first page (page 1) with page size 5
    let page1_result = user_repository::search(
        &context,
        &UserSearchParams {
            email: Some("pagination_test"),
            page: Some(1),
            page_size: Some(5),
            order_by: Some(&order_by),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(page1_result.len(), 5);

    // Test second page (page 2) with page size 5
    let page2_result = user_repository::search(
        &context,
        &UserSearchParams {
            email: Some("pagination_test"),
            page: Some(2),
            page_size: Some(5),
            order_by: Some(&order_by),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(page2_result.len(), 5);

    // Test third page (page 3) with page size 5
    let page3_result = user_repository::search(
        &context,
        &UserSearchParams {
            email: Some("pagination_test"),
            page: Some(3),
            page_size: Some(5),
            order_by: Some(&order_by),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(page3_result.len(), 5);

    // Test fourth page (page 4) with page size 5 - should have no results or fewer
    let page4_result = user_repository::search(
        &context,
        &UserSearchParams {
            email: Some("pagination_test"),
            page: Some(4),
            page_size: Some(5),
            order_by: Some(&order_by),
            ..Default::default()
        },
    )
    .await?;

    assert!(page4_result.len() <= 5);

    // Ensure pages contain different users
    let page1_emails: std::collections::HashSet<_> =
        page1_result.iter().map(|u| &u.email).collect();
    let page2_emails: std::collections::HashSet<_> =
        page2_result.iter().map(|u| &u.email).collect();

    assert!(page1_emails.is_disjoint(&page2_emails));

    Ok(())
}

#[tokio::test]
async fn test_search_empty_result() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Search for non-existent data
    let result = user_repository::search(
        &context,
        &UserSearchParams {
            email: Some("nonexistent"),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_update_user_all_fields() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create initial user
    let initial_user = user::ActiveModel {
        email: Set("update_test@example.com".to_string()),
        password: Set("old_password".to_string()),
        first_name: Set(Some("Old".to_string())),
        last_name: Set(Some("Name".to_string())),
        phone: Set(Some("123-456-7890".to_string())),
        ..Default::default()
    };

    let created_user = user_repository::create(&context, initial_user).await?;
    let initial_updated_at = created_user.updated_at;

    // Update all fields
    let mut user_active: user::ActiveModel = created_user.clone().into();
    user_active.email = Set("new_email@example.com".to_string());
    user_active.password = Set("new_password".to_string());
    user_active.first_name = Set(Some("New".to_string()));
    user_active.last_name = Set(Some("Updated".to_string()));
    user_active.phone = Set(Some("098-765-4321".to_string()));

    let updated_user = user_repository::update(&context, user_active).await?;

    // Verify all fields were updated
    assert_eq!(updated_user.email, "new_email@example.com");
    assert_eq!(updated_user.password, "new_password");
    assert_eq!(updated_user.first_name.unwrap(), "New");
    assert_eq!(updated_user.last_name.unwrap(), "Updated");
    assert_eq!(updated_user.phone.unwrap(), "098-765-4321");

    // Verify updated_at was changed
    assert_ne!(updated_user.updated_at, initial_updated_at);
    assert!(updated_user.updated_at > initial_updated_at);

    Ok(())
}

#[tokio::test]
async fn test_update_user_partial_fields() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create initial user
    let initial_user = user::ActiveModel {
        email: Set("partial_update@example.com".to_string()),
        password: Set("original_password".to_string()),
        first_name: Set(Some("Original".to_string())),
        last_name: Set(Some("LastName".to_string())),
        phone: Set(Some("111-222-3333".to_string())),
        ..Default::default()
    };

    let created_user = user_repository::create(&context, initial_user).await?;

    // Update only email and first_name
    let mut user_active: user::ActiveModel = created_user.clone().into();
    user_active.email = Set("updated_email@example.com".to_string());
    user_active.first_name = Set(Some("Updated".to_string()));

    let updated_user = user_repository::update(&context, user_active).await?;

    // Verify only specified fields were updated
    assert_eq!(updated_user.email, "updated_email@example.com");
    assert_eq!(updated_user.password, "original_password"); // unchanged
    assert_eq!(updated_user.first_name.unwrap(), "Updated");
    assert_eq!(updated_user.last_name.unwrap(), "LastName"); // unchanged
    assert_eq!(updated_user.phone.unwrap(), "111-222-3333"); // unchanged
    assert_eq!(updated_user.updated_user_id, created_user.updated_user_id); // unchanged

    Ok(())
}

#[tokio::test]
async fn test_update_user_not_found() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Try to update non-existent user
    let user_active = user::ActiveModel {
        id: Set(999999), // non-existent ID
        email: Set("test@example.com".to_string()),
        ..Default::default()
    };
    let result = user_repository::update(&context, user_active).await;

    // Should return an error when trying to update non-existent user
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_update_user_no_changes() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create initial user
    let initial_user = user::ActiveModel {
        email: Set("no_changes@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("NoChange".to_string())),
        last_name: Set(Some("User".to_string())),
        phone: Set(Some("555-123-4567".to_string())),
        ..Default::default()
    };

    let created_user = user_repository::create(&context, initial_user).await?;
    let initial_updated_at = created_user.updated_at;

    // Update with no field changes (just convert existing user to ActiveModel)
    let user_active: user::ActiveModel = created_user.clone().into();
    let updated_user = user_repository::update(&context, user_active).await?;

    // Verify all original fields remain the same except updated_at
    assert_eq!(updated_user.email, created_user.email);
    assert_eq!(updated_user.password, created_user.password);
    assert_eq!(updated_user.first_name, created_user.first_name);
    assert_eq!(updated_user.last_name, created_user.last_name);
    assert_eq!(updated_user.phone, created_user.phone);
    assert_eq!(updated_user.updated_user_id, created_user.updated_user_id);

    // updated_at should still be updated even with no field changes
    assert!(updated_user.updated_at > initial_updated_at);

    Ok(())
}

#[tokio::test]
async fn test_update_user_clear_optional_fields() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    // Create initial user with optional fields
    let initial_user = user::ActiveModel {
        email: Set("clear_fields@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Clear".to_string())),
        last_name: Set(Some("Fields".to_string())),
        phone: Set(Some("999-888-7777".to_string())),
        ..Default::default()
    };

    let created_user = user_repository::create(&context, initial_user).await?;

    // Update with empty strings to simulate clearing fields
    let mut user_active: user::ActiveModel = created_user.clone().into();
    user_active.first_name = Set(Some("".to_string())); // empty first_name
    user_active.last_name = Set(Some("".to_string())); // empty last_name
    user_active.phone = Set(Some("".to_string())); // empty phone

    let updated_user = user_repository::update(&context, user_active).await?;

    // Verify optional fields were updated to empty strings
    assert_eq!(updated_user.first_name.unwrap(), "");
    assert_eq!(updated_user.last_name.unwrap(), "");
    assert_eq!(updated_user.phone.unwrap(), "");

    Ok(())
}

#[tokio::test]
async fn test_search_by_ids() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    let mut created_ids = Vec::new();
    for i in 1..=3 {
        let user = user::ActiveModel {
            email: Set(format!("ids_test_{}@example.com", i)),
            password: Set("password123@".to_string()),
            first_name: Set(Some(format!("User{}", i))),
            last_name: Set(Some("IdsTest".to_string())),
            ..Default::default()
        };
        let created = user_repository::create(&context, user).await?;
        created_ids.push(created.id);
    }

    // Search by specific ids
    let search_ids = vec![created_ids[0], created_ids[2]];
    let result = user_repository::search(
        &context,
        &UserSearchParams {
            ids: Some(&search_ids),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 2);
    let result_ids: Vec<i32> = result.iter().map(|u| u.id).collect();
    assert!(result_ids.contains(&created_ids[0]));
    assert!(result_ids.contains(&created_ids[2]));

    Ok(())
}

#[tokio::test]
async fn test_search_with_ordering_by_email() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    for name in ["charlie", "alice", "bob"] {
        let user = user::ActiveModel {
            email: Set(format!("{}_order_test@example.com", name)),
            password: Set("password123@".to_string()),
            first_name: Set(Some(name.to_string())),
            last_name: Set(Some("OrderTest".to_string())),
            ..Default::default()
        };
        user_repository::create(&context, user).await?;
    }

    let order_by = vec![UserOrderBy {
        field: UserOrderByField::Email,
        order: SortOrder::Asc,
    }];

    let result = user_repository::search(
        &context,
        &UserSearchParams {
            last_name: Some("OrderTest"),
            order_by: Some(&order_by),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 3);
    assert!(result[0].email.contains("alice"));
    assert!(result[1].email.contains("bob"));
    assert!(result[2].email.contains("charlie"));

    Ok(())
}

#[tokio::test]
async fn test_search_with_ordering_by_first_name_desc() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    for (i, name) in ["Alpha", "Gamma", "Beta"].iter().enumerate() {
        let user = user::ActiveModel {
            email: Set(format!("fname_order_{}@example.com", i)),
            password: Set("password123@".to_string()),
            first_name: Set(Some(name.to_string())),
            last_name: Set(Some("FnameOrder".to_string())),
            ..Default::default()
        };
        user_repository::create(&context, user).await?;
    }

    let order_by = vec![UserOrderBy {
        field: UserOrderByField::FirstName,
        order: SortOrder::Desc,
    }];

    let result = user_repository::search(
        &context,
        &UserSearchParams {
            last_name: Some("FnameOrder"),
            order_by: Some(&order_by),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].first_name.as_deref(), Some("Gamma"));
    assert_eq!(result[1].first_name.as_deref(), Some("Beta"));
    assert_eq!(result[2].first_name.as_deref(), Some("Alpha"));

    Ok(())
}

#[tokio::test]
async fn test_search_with_ordering_by_last_name() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    for (i, lname) in ["Zulu", "Alpha", "Mike"].iter().enumerate() {
        let user = user::ActiveModel {
            email: Set(format!("lname_order_{}@example.com", i)),
            password: Set("password123@".to_string()),
            first_name: Set(Some("Same".to_string())),
            last_name: Set(Some(lname.to_string())),
            ..Default::default()
        };
        user_repository::create(&context, user).await?;
    }

    let order_by = vec![UserOrderBy {
        field: UserOrderByField::LastName,
        order: SortOrder::Asc,
    }];

    let result = user_repository::search(
        &context,
        &UserSearchParams {
            first_name: Some("Same"),
            order_by: Some(&order_by),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].last_name.as_deref(), Some("Alpha"));
    assert_eq!(result[1].last_name.as_deref(), Some("Mike"));
    assert_eq!(result[2].last_name.as_deref(), Some("Zulu"));

    Ok(())
}

#[tokio::test]
async fn test_search_with_ordering_by_created_at() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    for i in 1..=3 {
        let user = user::ActiveModel {
            email: Set(format!("created_order_{}@example.com", i)),
            password: Set("password123@".to_string()),
            first_name: Set(Some(format!("User{}", i))),
            last_name: Set(Some("CreatedOrder".to_string())),
            ..Default::default()
        };
        user_repository::create(&context, user).await?;
    }

    let order_by = vec![UserOrderBy {
        field: UserOrderByField::CreatedAt,
        order: SortOrder::Desc,
    }];

    let result = user_repository::search(
        &context,
        &UserSearchParams {
            last_name: Some("CreatedOrder"),
            order_by: Some(&order_by),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_search_with_ordering_by_updated_at() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    for i in 1..=2 {
        let user = user::ActiveModel {
            email: Set(format!("updated_order_{}@example.com", i)),
            password: Set("password123@".to_string()),
            first_name: Set(Some(format!("User{}", i))),
            last_name: Set(Some("UpdatedOrder".to_string())),
            ..Default::default()
        };
        user_repository::create(&context, user).await?;
    }

    let order_by = vec![UserOrderBy {
        field: UserOrderByField::UpdatedAt,
        order: SortOrder::Asc,
    }];

    let result = user_repository::search(
        &context,
        &UserSearchParams {
            last_name: Some("UpdatedOrder"),
            order_by: Some(&order_by),
            ..Default::default()
        },
    )
    .await?;

    assert_eq!(result.len(), 2);

    Ok(())
}

#[test]
fn test_user_order_by_field_from_str() {
    assert_eq!(UserOrderByField::from_str("id"), Some(UserOrderByField::Id));
    assert_eq!(
        UserOrderByField::from_str("email"),
        Some(UserOrderByField::Email)
    );
    assert_eq!(
        UserOrderByField::from_str("first_name"),
        Some(UserOrderByField::FirstName)
    );
    assert_eq!(
        UserOrderByField::from_str("last_name"),
        Some(UserOrderByField::LastName)
    );
    assert_eq!(
        UserOrderByField::from_str("created_at"),
        Some(UserOrderByField::CreatedAt)
    );
    assert_eq!(
        UserOrderByField::from_str("updated_at"),
        Some(UserOrderByField::UpdatedAt)
    );
    assert_eq!(UserOrderByField::from_str("unknown"), None);
    assert_eq!(UserOrderByField::from_str(""), None);
}

#[test]
fn test_user_order_by_field_to_string() {
    use my_axum::core::db::ordering::OrderByField;
    assert_eq!(UserOrderByField::Id.to_string(), "id");
    assert_eq!(UserOrderByField::Email.to_string(), "email");
    assert_eq!(UserOrderByField::FirstName.to_string(), "first_name");
    assert_eq!(UserOrderByField::LastName.to_string(), "last_name");
    assert_eq!(UserOrderByField::CreatedAt.to_string(), "created_at");
    assert_eq!(UserOrderByField::UpdatedAt.to_string(), "updated_at");
}

#[tokio::test]
async fn test_delete_by_id() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;
    let context = Context::builder(Arc::new(txn)).build();

    let user = user::ActiveModel {
        email: Set("delete_by_id@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Delete".to_string())),
        last_name: Set(Some("ById".to_string())),
        ..Default::default()
    };
    let created = user_repository::create(&context, user).await?;
    let id = created.id;

    user_repository::delete_by_id(&context, id).await?;

    let result = user_repository::find_by_id(&context, id).await?;
    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_create_and_update_with_user_context() -> Result<(), DbErr> {
    let test_app = TestApp::spawn_app().await;
    let txn = test_app.begin_transaction().await;

    // First create a user without context user
    let context = Context::builder(Arc::new(txn)).build();
    let user = user::ActiveModel {
        email: Set("context_user_test@example.com".to_string()),
        password: Set("password123@".to_string()),
        first_name: Set(Some("Context".to_string())),
        last_name: Set(Some("Test".to_string())),
        ..Default::default()
    };
    let created = user_repository::create(&context, user).await?;
    assert!(created.created_user_id.is_none());

    // Now create a context with a user set, and update
    let mut context = context;
    context.user = Some(created.clone());

    let mut active: user::ActiveModel = created.clone().into();
    active.first_name = Set(Some("Updated".to_string()));
    let updated = user_repository::update(&context, active).await?;

    assert_eq!(updated.first_name, Some("Updated".to_string()));
    assert_eq!(updated.updated_user_id, Some(created.id));

    Ok(())
}
