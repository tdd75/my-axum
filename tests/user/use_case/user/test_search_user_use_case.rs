use my_axum::user::dto::user_dto::{UserCreateDTO, UserSearchParamsDTO};
use my_axum::user::use_case::user::search_user_use_case;
use sea_orm::DbErr;

use crate::setup::app::TestApp;

mod search_user_tests {
    use my_axum::{core::context::Context, user::use_case::user::create_user_use_case};

    use super::*;

    #[tokio::test]
    async fn should_search_users_without_filters() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create multiple users
        let users = vec![
            UserCreateDTO {
                email: "search1@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alice".to_string()),
                last_name: Some("Johnson".to_string()),
                phone: Some("123-456-7890".to_string()),
            },
            UserCreateDTO {
                email: "search2@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Bob".to_string()),
                last_name: Some("Smith".to_string()),
                phone: Some("123-456-7891".to_string()),
            },
            UserCreateDTO {
                email: "search3@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Charlie".to_string()),
                last_name: Some("Brown".to_string()),
                phone: Some("123-456-7892".to_string()),
            },
        ];

        for user_dto in users {
            create_user_use_case::execute(&context, user_dto)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
        }

        // Search without filters
        let search_param = UserSearchParamsDTO {
            email: None,
            first_name: None,
            last_name: None,
            page: None,
            page_size: None,
            order_by: None,
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert!(result.data.items.len() >= 3);
        assert_eq!(result.data.count, result.data.items.len());

        Ok(())
    }

    #[tokio::test]
    async fn should_search_users_by_email() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create users with specific emails
        let users = vec![
            UserCreateDTO {
                email: "email_search_test1@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alice".to_string()),
                last_name: Some("Johnson".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "email_search_test2@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Bob".to_string()),
                last_name: Some("Smith".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "other@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Charlie".to_string()),
                last_name: Some("Brown".to_string()),
                phone: None,
            },
        ];

        for user_dto in users {
            create_user_use_case::execute(&context, user_dto)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
        }

        // Search by email
        let search_param = UserSearchParamsDTO {
            email: Some("email_search_test".to_string()),
            first_name: None,
            last_name: None,
            page: None,
            page_size: None,
            order_by: None,
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(result.data.items.len(), 2);
        assert_eq!(result.data.count, 2);
        assert!(
            result
                .data
                .items
                .iter()
                .all(|u| u.email.contains("email_search_test"))
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_search_users_by_first_name() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create users with specific first names
        let users = vec![
            UserCreateDTO {
                email: "fname_test1@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alexander".to_string()),
                last_name: Some("Johnson".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "fname_test2@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alexandra".to_string()),
                last_name: Some("Smith".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "fname_test3@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Bob".to_string()),
                last_name: Some("Brown".to_string()),
                phone: None,
            },
        ];

        for user_dto in users {
            create_user_use_case::execute(&context, user_dto)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
        }

        // Search by first name
        let search_param = UserSearchParamsDTO {
            email: None,
            first_name: Some("Alex".to_string()),
            last_name: None,
            page: None,
            page_size: None,
            order_by: None,
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(result.data.items.len(), 2);
        assert_eq!(result.data.count, 2);
        assert!(
            result
                .data
                .items
                .iter()
                .all(|u| { u.first_name.as_ref().unwrap().contains("Alex") })
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_search_users_by_last_name() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create users with specific last names
        let users = vec![
            UserCreateDTO {
                email: "lname_test1@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alice".to_string()),
                last_name: Some("Johnson".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "lname_test2@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Bob".to_string()),
                last_name: Some("Johnson".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "lname_test3@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Charlie".to_string()),
                last_name: Some("Smith".to_string()),
                phone: None,
            },
        ];

        for user_dto in users {
            create_user_use_case::execute(&context, user_dto)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
        }

        // Search by last name
        let search_param = UserSearchParamsDTO {
            email: None,
            first_name: None,
            last_name: Some("Johnson".to_string()),
            page: None,
            page_size: None,
            order_by: None,
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(result.data.items.len(), 2);
        assert_eq!(result.data.count, 2);
        assert!(
            result
                .data
                .items
                .iter()
                .all(|u| { u.last_name.as_ref().unwrap().contains("Johnson") })
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_search_users_with_multiple_filters() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

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
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

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
        assert_eq!(page1_result.data.count, 5);

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
        assert_eq!(page2_result.data.count, 5);

        // Ensure pages contain different users
        let page1_emails: std::collections::HashSet<_> =
            page1_result.data.items.iter().map(|u| &u.email).collect();
        let page2_emails: std::collections::HashSet<_> =
            page2_result.data.items.iter().map(|u| &u.email).collect();

        assert!(page1_emails.is_disjoint(&page2_emails));

        Ok(())
    }

    #[tokio::test]
    async fn should_return_empty_result_when_no_matches() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Search for non-existent data
        let search_param = UserSearchParamsDTO {
            email: Some("nonexistent_search_term".to_string()),
            first_name: None,
            last_name: None,
            page: None,
            page_size: None,
            order_by: None,
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(result.data.items.len(), 0);
        assert_eq!(result.data.count, 0);

        Ok(())
    }

    #[tokio::test]
    async fn should_handle_default_pagination_params() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create a few users
        for i in 1..=3 {
            let user_dto = UserCreateDTO {
                email: format!("default_pagination_{}@example.com", i),
                password: "password123@".to_string(),
                first_name: Some(format!("User{}", i)),
                last_name: Some("Test".to_string()),
                phone: None,
            };
            create_user_use_case::execute(&context, user_dto)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
        }

        // Search without pagination params (should use defaults)
        let search_param = UserSearchParamsDTO {
            email: Some("default_pagination".to_string()),
            first_name: None,
            last_name: None,
            page: None,
            page_size: None,
            order_by: None,
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(result.data.items.len(), 3);
        assert_eq!(result.data.count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn should_search_users_with_order_by_ascending() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create users with specific timestamps
        let users = vec![
            UserCreateDTO {
                email: "order_test_1@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Charlie".to_string()),
                last_name: Some("Brown".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "order_test_2@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alice".to_string()),
                last_name: Some("Johnson".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "order_test_3@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Bob".to_string()),
                last_name: Some("Smith".to_string()),
                phone: None,
            },
        ];

        for user_dto in users {
            create_user_use_case::execute(&context, user_dto)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
        }

        // Search ordered by first_name ascending
        let search_param = UserSearchParamsDTO {
            email: Some("order_test".to_string()),
            first_name: None,
            last_name: None,
            page: None,
            page_size: None,
            order_by: Some("+first_name".to_string()),
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(result.data.items.len(), 3);
        assert_eq!(result.data.items[0].first_name, Some("Alice".to_string()));
        assert_eq!(result.data.items[1].first_name, Some("Bob".to_string()));
        assert_eq!(result.data.items[2].first_name, Some("Charlie".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn should_search_users_with_order_by_descending() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create users with specific IDs
        let users = vec![
            UserCreateDTO {
                email: "desc_order_test_1@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Alice".to_string()),
                last_name: Some("Johnson".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "desc_order_test_2@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Bob".to_string()),
                last_name: Some("Smith".to_string()),
                phone: None,
            },
            UserCreateDTO {
                email: "desc_order_test_3@example.com".to_string(),
                password: "password123@".to_string(),
                first_name: Some("Charlie".to_string()),
                last_name: Some("Brown".to_string()),
                phone: None,
            },
        ];

        for user_dto in users {
            create_user_use_case::execute(&context, user_dto)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
        }

        // Search ordered by id descending
        let search_param = UserSearchParamsDTO {
            email: Some("desc_order_test".to_string()),
            first_name: None,
            last_name: None,
            page: None,
            page_size: None,
            order_by: Some("-id".to_string()),
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        assert_eq!(result.data.items.len(), 3);
        // When ordered by ID desc, the last created user should be first
        let first_id = result.data.items[0].id;
        let second_id = result.data.items[1].id;
        let third_id = result.data.items[2].id;

        assert!(first_id > second_id);
        assert!(second_id > third_id);

        Ok(())
    }

    #[tokio::test]
    async fn should_search_users_with_multiple_order_by_fields() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

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

    #[tokio::test]
    async fn should_search_users_with_invalid_order_by_field() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        // Create a user
        let user_dto = UserCreateDTO {
            email: "invalid_order_test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };

        create_user_use_case::execute(&context, user_dto)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        // Search with invalid order field (should be ignored)
        let search_param = UserSearchParamsDTO {
            email: Some("invalid_order_test".to_string()),
            first_name: None,
            last_name: None,
            page: None,
            page_size: None,
            order_by: Some("+invalid_field".to_string()),
        };

        let result = search_user_use_case::execute(&context, search_param)
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        // Should still return results even with invalid order field
        assert_eq!(result.data.items.len(), 1);
        assert_eq!(result.data.items[0].email, "invalid_order_test@example.com");

        Ok(())
    }
}
