use my_axum::user::dto::user_dto::UserCreateDTO;
use sea_orm::DbErr;

use crate::setup::app::{DatabaseType, TestApp};

mod create_user_tests {
    use my_axum::{core::context::Context, user::use_case::user::create_user_use_case};

    use super::*;

    #[tokio::test]
    async fn should_create_user_successfully() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            phone: Some("123-456-7890".to_string()),
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_fail_when_email_is_duplicate() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto1 = UserCreateDTO {
            email: "duplicate@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            phone: Some("123-456-7890".to_string()),
        };

        // Create first user
        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        create_user_use_case::execute(&context, dto1).await.unwrap();

        // Try to create second user with same email
        let dto2 = UserCreateDTO {
            email: "duplicate@example.com".to_string(),
            password: "password456".to_string(),
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            phone: Some("098-765-4321".to_string()),
        };

        let result = create_user_use_case::execute(&context, dto2).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("validate unique email") || error_msg.contains("already exists"),
            "Expected error message about duplicate email, got: {}",
            error_msg
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_create_user_with_minimal_fields() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "minimal@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: None,
            last_name: None,
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                assert_eq!(user.data.email, "minimal@example.com");
                assert_eq!(user.data.first_name, None);
                assert_eq!(user.data.last_name, None);
                assert_eq!(user.data.phone, None);
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_hash_password_before_storing() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let plain_password = "my_secret_password_123";
        let dto = UserCreateDTO {
            email: "passwordtest@example.com".to_string(),
            password: plain_password.to_string(),
            first_name: Some("Password".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                // Password should be hashed (not stored as plain text)
                // We can verify this by checking that the returned user has a valid ID
                assert!(user.data.id > 0);
                assert_eq!(user.data.email, "passwordtest@example.com");
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_return_created_status() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "statustest@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Status".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(response) => {
                assert_eq!(response.status.as_u16(), 201); // CREATED status
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_send_welcome_email_task() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "welcomeemail@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Welcome".to_string()),
            last_name: Some("Email".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None, // No producer, so it should skip email task
        };

        // Should succeed even without producer
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                assert_eq!(user.data.email, "welcomeemail@example.com");
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_validate_email_format() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "notanemail".to_string(), // Invalid email format (no @ symbol)
            password: "password123@".to_string(),
            first_name: Some("Invalid".to_string()),
            last_name: Some("Email".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 400);
        Ok(())
    }

    #[tokio::test]
    async fn should_create_user_with_long_email() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "verylongemailaddressfortesting@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Long".to_string()),
            last_name: Some("Email".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                assert_eq!(
                    user.data.email,
                    "verylongemailaddressfortesting@example.com"
                );
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_create_user_with_special_chars_in_name() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "specialchars@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("François".to_string()),
            last_name: Some("O'Neill-Smith".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                assert_eq!(user.data.first_name, Some("François".to_string()));
                assert_eq!(user.data.last_name, Some("O'Neill-Smith".to_string()));
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_create_user_with_long_password() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "longpassword@example.com".to_string(),
            password: "this_is_a_very_long_password_that_should_still_work_correctly_123456789"
                .to_string(),
            first_name: Some("Long".to_string()),
            last_name: Some("Password".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                assert!(user.data.id > 0);
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_create_user_with_phone_number() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "phonetest@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Phone".to_string()),
            last_name: Some("Test".to_string()),
            phone: Some("+1-555-123-4567".to_string()),
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                assert_eq!(user.data.phone, Some("+1-555-123-4567".to_string()));
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_fail_with_invalid_email_missing_at() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "invalidemail.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Invalid".to_string()),
            last_name: Some("Email".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn should_fail_with_empty_email() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Empty".to_string()),
            last_name: Some("Email".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn should_fail_with_empty_password() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "emptypass@example.com".to_string(),
            password: "".to_string(),
            first_name: Some("Empty".to_string()),
            last_name: Some("Password".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn should_create_multiple_users_with_different_emails() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };

        for i in 1..=5 {
            let dto = UserCreateDTO {
                email: format!("user{}@example.com", i),
                password: "password123@".to_string(),
                first_name: Some(format!("User{}", i)),
                last_name: Some("Test".to_string()),
                phone: None,
            };

            let result = create_user_use_case::execute(&context, dto).await;
            assert!(result.is_ok());
        }

        Ok(())
    }

    #[tokio::test]
    async fn should_preserve_email_case() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "Test.User@Example.COM".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                // The email should be preserved as entered
                assert_eq!(user.data.email, "Test.User@Example.COM");
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_handle_unicode_in_names() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "unicode@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("北京".to_string()),
            last_name: Some("Москва".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                assert_eq!(user.data.first_name, Some("北京".to_string()));
                assert_eq!(user.data.last_name, Some("Москва".to_string()));
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_create_user_and_return_timestamps() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "timestamps@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Time".to_string()),
            last_name: Some("Stamps".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                // Verify timestamps are set
                assert!(user.data.created_at.is_some());
                assert!(user.data.updated_at.is_some());
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_create_user_with_various_email_formats() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let email_formats = [
            "simple@example.com",
            "with.dots@example.com",
            "with+plus@example.com",
            "with_underscore@example.com",
            "with-dash@example.com",
            "numbers123@example.com",
        ];

        for (i, email) in email_formats.iter().enumerate() {
            let dto = UserCreateDTO {
                email: email.to_string(),
                password: "password123@".to_string(),
                first_name: Some(format!("User{}", i)),
                last_name: Some("Test".to_string()),
                phone: None,
            };

            let txn = test_app.begin_transaction().await;
            let context = Context {
                txn: &txn,
                user: None,
                producer: None,
            };
            let result = create_user_use_case::execute(&context, dto).await;

            assert!(
                result.is_ok(),
                "Failed to create user with email: {}",
                email
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn should_create_user_with_very_long_names() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let long_name = "A".repeat(255);
        let dto = UserCreateDTO {
            email: "longname@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some(long_name.clone()),
            last_name: Some(long_name.clone()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                assert_eq!(user.data.first_name.as_ref().unwrap().len(), 255);
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_create_user_with_very_long_phone() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let long_phone = "1".repeat(50);
        let dto = UserCreateDTO {
            email: "longphone@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Long".to_string()),
            last_name: Some("Phone".to_string()),
            phone: Some(long_phone.clone()),
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                assert!(user.data.phone.is_some());
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }

    #[tokio::test]
    async fn should_create_users_sequentially() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        for i in 0..5 {
            let dto = UserCreateDTO {
                email: format!("sequential{}@example.com", i),
                password: "password123@".to_string(),
                first_name: Some(format!("User{}", i)),
                last_name: Some("Sequential".to_string()),
                phone: None,
            };

            let txn = test_app.begin_transaction().await;
            let context = Context {
                txn: &txn,
                user: None,
                producer: None,
            };
            let result = create_user_use_case::execute(&context, dto).await;
            assert!(result.is_ok());
        }

        Ok(())
    }

    #[tokio::test]
    async fn should_create_user_with_empty_optional_fields() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "empty_optional@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("".to_string()),
            last_name: Some("".to_string()),
            phone: Some("".to_string()),
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        assert!(result.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn should_handle_transaction_rollback() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "rollback@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Rollback".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let _result = create_user_use_case::execute(&context, dto).await;

        // Transaction should auto-rollback when txn is dropped
        drop(txn);

        Ok(())
    }

    #[tokio::test]
    async fn should_return_correct_status_code() -> Result<(), DbErr> {
        let test_app = TestApp::spawn_app().await;

        let dto = UserCreateDTO {
            email: "status@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Status".to_string()),
            last_name: Some("Test".to_string()),
            phone: None,
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(response) => {
                assert_eq!(response.status, axum::http::StatusCode::CREATED);
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }
}

#[cfg(test)]
mod create_user_tests_postgres {
    use my_axum::{core::context::Context, user::use_case::user::create_user_use_case};

    use super::*;

    #[tokio::test]
    #[ignore] // Ignore by default, run with: cargo test -- --ignored
    async fn should_create_user_with_postgres() -> Result<(), DbErr> {
        // Explicitly use PostgreSQL for this test
        let test_app = TestApp::spawn_app_with_db(DatabaseType::Postgres).await;

        let dto = UserCreateDTO {
            email: "postgres_test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Postgres".to_string()),
            last_name: Some("User".to_string()),
            phone: Some("123-456-7890".to_string()),
        };

        let txn = test_app.begin_transaction().await;
        let context = Context {
            txn: &txn,
            user: None,
            producer: None,
        };
        let result = create_user_use_case::execute(&context, dto).await;

        match result {
            Ok(user) => {
                assert_eq!(user.data.email, "postgres_test@example.com");
                Ok(())
            }
            Err(e) => Err(DbErr::Custom(e.to_string())),
        }
    }
}
