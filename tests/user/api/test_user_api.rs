mod create_user_tests {
    use my_axum::core::context::Context;
    use reqwest::Client;
    use sea_orm::{DbErr, TransactionTrait};
    use serde_json::{Value, json};

    use crate::setup::{app::TestApp, fixture::login_normal_user};

    #[tokio::test]
    async fn test_create_user() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        let access_token = test_app
            .db
            .transaction::<_, String, DbErr>(|txn| {
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

        let payload = json!({
            "email": "john.doe@example.com".to_string(),
            "password": "password",
            "first_name": "John".to_string(),
            "last_name": "Doe".to_string(),
        });

        // Act
        let response = client
            .post(format!("http://{}/api/v1/user/", &test_app.base_url))
            .json(&payload)
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();

        // Assert
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap();
            panic!(
                "Request failed with status: {}, body: {}",
                status, error_body
            );
        }
        let result = &response.json::<Value>().await.unwrap();

        assert!(result.get("id").unwrap().as_i64().unwrap() > 0);
        assert_eq!(
            result.get("email").unwrap().as_str().unwrap(),
            payload.get("email").unwrap().as_str().unwrap(),
        );
        assert!(result.get("password").is_none());
        assert_eq!(
            result.get("first_name").unwrap().as_str().unwrap(),
            payload.get("first_name").unwrap().as_str().unwrap(),
        );
        assert_eq!(
            result.get("last_name").unwrap().as_str().unwrap(),
            payload.get("last_name").unwrap().as_str().unwrap(),
        );
    }
}

mod get_user_tests {
    use my_axum::core::context::Context;
    use my_axum::user::use_case::user::create_user_use_case;
    use reqwest::{Client, StatusCode};
    use sea_orm::{DbErr, TransactionTrait};
    use serde_json::Value;

    use crate::{setup::app::TestApp, setup::fixture::login_normal_user};
    use my_axum::user::dto::user_dto::UserCreateDTO;
    use my_axum::user::dto::user_dto::UserDTO;

    #[tokio::test]
    async fn test_get_user_success() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        let (access_token, user) = test_app
            .db
            .transaction::<_, (String, UserDTO), DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _) = login_normal_user(&mut context).await;
                    let dto = UserCreateDTO {
                        email: "get.user@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let user = create_user_use_case::execute(&context, dto)
                        .await
                        .unwrap()
                        .data;
                    Ok((access_token, user))
                })
            })
            .await
            .unwrap();

        // Act
        let response = client
            .get(format!(
                "http://{}/api/v1/user/{}/",
                &test_app.base_url, user.id
            ))
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let result = &response.json::<Value>().await.unwrap();
        assert_eq!(result.get("id").unwrap().as_i64().unwrap(), user.id as i64);
        assert_eq!(result.get("email").unwrap().as_str().unwrap(), user.email);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let non_existent_id = 9999;

        let access_token = test_app
            .db
            .transaction::<_, String, DbErr>(|txn| {
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

        // Act
        let response = client
            .get(format!(
                "http://{}/api/v1/user/{}/",
                &test_app.base_url, non_existent_id
            ))
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

mod delete_user_tests {
    use reqwest::{Client, StatusCode};
    use sea_orm::{DbErr, TransactionTrait};

    use crate::setup::{app::TestApp, fixture::login_normal_user};
    use my_axum::{
        core::context::Context,
        user::{
            dto::user_dto::UserCreateDTO, repository::user_repository,
            use_case::user::create_user_use_case,
        },
    };

    #[tokio::test]
    async fn test_delete_user_success() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        let (access_token, user_id) = test_app
            .db
            .transaction::<_, (String, i32), DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _) = login_normal_user(&mut context).await;
                    let dto = UserCreateDTO {
                        email: "delete.me@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let user_to_delete = create_user_use_case::execute(&context, dto)
                        .await
                        .unwrap()
                        .data;
                    Ok((access_token, user_to_delete.id))
                })
            })
            .await
            .unwrap();

        // Act
        let response = client
            .delete(format!(
                "http://{}/api/v1/user/{}/",
                &test_app.base_url, user_id
            ))
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let new_txn = test_app.begin_transaction().await;
        let new_context = Context {
            txn: &new_txn,
            user: None,
            producer: None,
        };
        let deleted_user = user_repository::find_by_id(&new_context, user_id)
            .await
            .unwrap();
        assert!(deleted_user.is_none());
    }
}

mod update_user_tests {
    use reqwest::{Client, StatusCode};
    use sea_orm::{DbErr, TransactionTrait};
    use serde_json::{Value, json};

    use crate::setup::{app::TestApp, fixture::login_normal_user};
    use my_axum::{
        core::context::Context,
        user::{dto::user_dto::UserCreateDTO, use_case::user::create_user_use_case},
    };

    #[tokio::test]
    async fn test_update_user_success() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        let (access_token, user_id) = test_app
            .db
            .transaction::<_, (String, i32), DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _) = login_normal_user(&mut context).await;
                    let dto = UserCreateDTO {
                        email: "update.me@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let user_to_update = create_user_use_case::execute(&context, dto)
                        .await
                        .unwrap()
                        .data;
                    Ok((access_token, user_to_update.id))
                })
            })
            .await
            .unwrap();

        let payload = json!({
            "first_name": "Updated",
            "last_name": "Name"
        });

        // Act
        let response = client
            .patch(format!(
                "http://{}/api/v1/user/{}/",
                &test_app.base_url, user_id
            ))
            .json(&payload)
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let result = &response.json::<Value>().await.unwrap();
        assert_eq!(
            result.get("first_name").unwrap().as_str().unwrap(),
            "Updated"
        );
        assert_eq!(result.get("last_name").unwrap().as_str().unwrap(), "Name");
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();
        let non_existent_id = 9999;

        let access_token = test_app
            .db
            .transaction::<_, String, DbErr>(|txn| {
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

        let payload = json!({
            "first_name": "Updated"
        });

        // Act
        let response = client
            .patch(format!(
                "http://{}/api/v1/user/{}/",
                &test_app.base_url, non_existent_id
            ))
            .json(&payload)
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

mod search_user_tests {
    use my_axum::{
        core::context::Context,
        user::{dto::user_dto::UserCreateDTO, use_case::user::create_user_use_case},
    };
    use reqwest::{Client, StatusCode};
    use sea_orm::{DbErr, TransactionTrait};
    use serde_json::Value;

    use crate::setup::{app::TestApp, fixture::login_normal_user};

    #[tokio::test]
    async fn test_search_user_by_email() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        let (access_token, user1_id) = test_app
            .db
            .transaction::<_, (String, i32), DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _) = login_normal_user(&mut context).await;
                    let dto1 = UserCreateDTO {
                        email: "search1@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let user1 = create_user_use_case::execute(&context, dto1)
                        .await
                        .unwrap()
                        .data;
                    let dto2 = UserCreateDTO {
                        email: "search2@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let _ = create_user_use_case::execute(&context, dto2)
                        .await
                        .unwrap()
                        .data;
                    Ok((access_token, user1.id))
                })
            })
            .await
            .unwrap();

        // Act
        let response = client
            .get(format!(
                "http://{}/api/v1/user/?email=search1@example.com",
                &test_app.base_url
            ))
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value = response.json().await.unwrap();
        let items = result.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].get("id").unwrap().as_i64().unwrap(),
            user1_id as i64
        );
    }

    #[tokio::test]
    async fn test_search_user_no_filters() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = Client::new();

        let access_token = test_app
            .db
            .transaction::<_, String, DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _user) = login_normal_user(&mut context).await;
                    let dto3 = UserCreateDTO {
                        email: "search3@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let _ = create_user_use_case::execute(&context, dto3)
                        .await
                        .unwrap()
                        .data;
                    let dto4 = UserCreateDTO {
                        email: "search4@example.com".to_string(),
                        password: "password123@".to_string(),
                        first_name: Some("Test".to_string()),
                        last_name: Some("User".to_string()),
                        phone: Some("1234567890".to_string()),
                    };
                    let _ = create_user_use_case::execute(&context, dto4)
                        .await
                        .unwrap()
                        .data;
                    Ok(access_token)
                })
            })
            .await
            .unwrap();

        // Act
        let response = client
            .get(format!("http://{}/api/v1/user/", &test_app.base_url))
            .bearer_auth(access_token)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value = response.json().await.unwrap();
        let items = result.get("items").unwrap().as_array().unwrap();
        // Includes the logged-in user + 2 created users
        assert_eq!(items.len(), 3);
        assert_eq!(result.get("count").unwrap().as_u64().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_search_user_with_order_by() {
        // Arrange
        let test_app = TestApp::spawn_app().await;
        let client = reqwest::Client::new();

        let access_token = test_app
            .db
            .transaction::<_, String, DbErr>(|txn| {
                Box::pin(async move {
                    let mut context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let (access_token, _) = login_normal_user(&mut context).await;

                    let test_users = vec![
                        UserCreateDTO {
                            email: "order_alice@example.com".to_string(),
                            password: "password123@".to_string(),
                            first_name: Some("Alice".to_string()),
                            last_name: Some("Smith".to_string()),
                            phone: None,
                        },
                        UserCreateDTO {
                            email: "order_bob@example.com".to_string(),
                            password: "password123@".to_string(),
                            first_name: Some("Bob".to_string()),
                            last_name: Some("Johnson".to_string()),
                            phone: None,
                        },
                        UserCreateDTO {
                            email: "order_charlie@example.com".to_string(),
                            password: "password123@".to_string(),
                            first_name: Some("Charlie".to_string()),
                            last_name: Some("Brown".to_string()),
                            phone: None,
                        },
                    ];

                    for user_dto in test_users {
                        create_user_use_case::execute(&context, user_dto)
                            .await
                            .unwrap();
                    }
                    Ok(access_token)
                })
            })
            .await
            .unwrap();

        // Act - Test ordering by first_name ascending
        let response_asc = client
            .get(format!(
                "http://{}/api/v1/user/?email=order_&order_by=+first_name",
                &test_app.base_url
            ))
            .bearer_auth(&access_token)
            .send()
            .await
            .unwrap();

        let response_desc = client
            .get(format!(
                "http://{}/api/v1/user/?email=order_&order_by=-first_name",
                &test_app.base_url
            ))
            .bearer_auth(&access_token)
            .send()
            .await
            .unwrap();

        // Assert
        assert_eq!(response_asc.status(), StatusCode::OK);
        let result_asc: Value = response_asc.json().await.unwrap();
        let items_asc = result_asc.get("items").unwrap().as_array().unwrap();
        assert_eq!(items_asc.len(), 3);

        let first_names_asc: Vec<&str> = items_asc
            .iter()
            .map(|item| item.get("first_name").unwrap().as_str().unwrap())
            .collect();
        assert_eq!(first_names_asc, vec!["Alice", "Bob", "Charlie"]);

        assert_eq!(response_desc.status(), StatusCode::OK);
        let result_desc: Value = response_desc.json().await.unwrap();
        let items_desc = result_desc.get("items").unwrap().as_array().unwrap();
        assert_eq!(items_desc.len(), 3);

        let first_names_desc: Vec<&str> = items_desc
            .iter()
            .map(|item| item.get("first_name").unwrap().as_str().unwrap())
            .collect();
        assert_eq!(first_names_desc, vec!["Charlie", "Bob", "Alice"]);
    }
}
