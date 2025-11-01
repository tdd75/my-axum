use my_axum::user::dto::user_dto::{UserCreateDTO, UserSearchParamsDTO};
use my_axum::user::use_case::user::search_user_use_case;
use sea_orm::DbErr;

use crate::setup::app::TestApp;

mod search_user_tests {
    use my_axum::{core::context::Context, user::use_case::user::create_user_use_case};
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn should_search_users_with_multiple_filters() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        // Create users for complex filtering
        let users = vec![
            UserCreateDTO {
                email: "multi_filter_alice@test.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alice".to_string()),
                last_name: Some("Johnson".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "multi_filter_bob@test.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alice".to_string()),
                last_name: Some("Smith".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "other_domain@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alice".to_string()),
                last_name: Some("Johnson".to_string()),
                phone: None,
            },
        ];

        for user_dto in users {
            create_user_use_case::execute(&context, user_dto)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
        }

        // Search with multiple filters
        let search_param = UserSearchParamsDTO {
            email: Some("test".to_string()),
            first_name: Some("Alice".to_string()),
            last_name: Some("Johnson".to_string()),
            page: None,
            page_size: None,
            order_by: None,
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(result.data.items.len(), 1);
        assert_eq!(result.data.count, 1);

        let user = &result.data.items[0];
        assert!(user.email.contains("test"));
        assert_eq!(user.first_name.as_ref().unwrap(), "Alice");
        assert_eq!(user.last_name.as_ref().unwrap(), "Johnson");

        Ok(())
    }

    #[tokio::test]
    async fn should_search_users_with_pagination() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        // Create multiple users for pagination testing
        for i in 1..=15 {
            let user_dto = UserCreateDTO {
                email: format!("pagination_use_case_{}@example.com", i),
                password: "password123@".to_string(),
                first_name: Some(format!("User{}", i)),
                last_name: Some("Test".to_string()),
                phone: None,
            };
            create_user_use_case::execute(&context, user_dto)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
        }

        // Test first page
        let search_param = UserSearchParamsDTO {
            email: Some("pagination_use_case".to_string()),
            first_name: None,
            last_name: None,
            page: Some(1),
            page_size: Some(5),
            order_by: Some("+id".to_string()), // Add consistent ordering
        };

        let page1_result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(page1_result.data.items.len(), 5);
        assert_eq!(page1_result.data.count, 15);

        // Test second page
        let search_param = UserSearchParamsDTO {
            email: Some("pagination_use_case".to_string()),
            first_name: None,
            last_name: None,
            page: Some(2),
            page_size: Some(5),
            order_by: Some("+id".to_string()), // Add consistent ordering
        };

        let page2_result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(page2_result.data.items.len(), 5);
        assert_eq!(page2_result.data.count, 15);

        // Ensure pages contain different users
        let page1_emails: std::collections::HashSet<_> =
            page1_result.data.items.iter().map(|u| &u.email).collect();
        let page2_emails: std::collections::HashSet<_> =
            page2_result.data.items.iter().map(|u| &u.email).collect();

        assert!(page1_emails.is_disjoint(&page2_emails));

        Ok(())
    }

    #[tokio::test]
    async fn should_search_users_with_multiple_order_by_fields() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        // Create users with same last name but different first names
        let users = vec![
            UserCreateDTO {
                email: "multi_order_1@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Bob".to_string()),
                last_name: Some("Smith".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "multi_order_2@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alice".to_string()),
                last_name: Some("Smith".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "multi_order_3@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Charlie".to_string()),
                last_name: Some("Johnson".to_string()),
                phone: None,
            },
        ];

        for user_dto in users {
            create_user_use_case::execute(&context, user_dto)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
        }

        // Search ordered by last_name ascending, then first_name descending
        let search_param = UserSearchParamsDTO {
            email: Some("multi_order".to_string()),
            first_name: None,
            last_name: None,
            page: None,
            page_size: None,
            order_by: Some("+last_name,-first_name".to_string()),
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(result.data.items.len(), 3);
        // Should be ordered by last_name asc, then first_name desc
        // Johnson comes before Smith, and within Smith, Bob comes before Alice (desc order)
        assert_eq!(result.data.items[0].last_name, Some("Johnson".to_string()));
        assert_eq!(result.data.items[0].first_name, Some("Charlie".to_string()));

        assert_eq!(result.data.items[1].last_name, Some("Smith".to_string()));
        assert_eq!(result.data.items[1].first_name, Some("Bob".to_string()));

        assert_eq!(result.data.items[2].last_name, Some("Smith".to_string()));
        assert_eq!(result.data.items[2].first_name, Some("Alice".to_string()));

        Ok(())
    }
}
