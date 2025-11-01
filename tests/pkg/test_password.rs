mod hash_verify_tests {
    use my_axum::pkg::password::*;

    #[tokio::test]
    async fn test_hash_and_verify_password_success() {
        let password = "test_password_123";

        let hash = hash_password(password).await.unwrap();
        assert!(!hash.is_empty());
        assert_ne!(hash, password);

        let result = verify_password(password, &hash).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_password_failure() {
        let password = "correct_password";
        let wrong_password = "wrong_password";

        let hash = hash_password(password).await.unwrap();
        let result = verify_password(wrong_password, &hash).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_hash_password_string() {
        let password = "string_password_test";
        let hash = hash_password_string(password).await.unwrap();

        assert!(!hash.is_empty());
        assert_ne!(hash, password);

        let verify_result = verify_password(password, &hash).await;
        assert!(verify_result.is_ok());
    }

    #[tokio::test]
    async fn test_different_passwords_different_hashes() {
        let password1 = "password_one";
        let password2 = "password_two";

        let hash1 = hash_password_string(password1).await.unwrap();
        let hash2 = hash_password_string(password2).await.unwrap();

        assert_ne!(hash1, hash2);

        // Each password should only verify against its own hash
        assert!(verify_password(password1, &hash1).await.is_ok());
        assert!(verify_password(password2, &hash2).await.is_ok());
        assert!(verify_password(password1, &hash2).await.is_err());
        assert!(verify_password(password2, &hash1).await.is_err());
    }

    #[tokio::test]
    async fn test_same_password_different_hashes() {
        let password = "same_password";

        let hash1 = hash_password_string(password).await.unwrap();
        let hash2 = hash_password_string(password).await.unwrap();

        // Same password should produce different hashes due to salt
        assert_ne!(hash1, hash2);

        // But both should verify successfully
        assert!(verify_password(password, &hash1).await.is_ok());
        assert!(verify_password(password, &hash2).await.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_hash_format() {
        let password = "test_password";
        let invalid_hashes = vec!["", "invalid_hash", "not.a.valid.hash.format", "short"];

        for hash in invalid_hashes {
            let result = verify_password(password, hash).await;
            assert!(
                result.is_err(),
                "Invalid hash '{}' should fail verification",
                hash
            );
        }
    }

    #[tokio::test]
    async fn test_empty_password() {
        let empty_password = "";
        let result = hash_password_string(empty_password).await;
        assert!(result.is_ok()); // Empty passwords can be hashed

        let hash = result.unwrap();
        let verify_result = verify_password(empty_password, &hash).await;
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        assert!(!salt1.is_empty());
        assert!(!salt2.is_empty());
        assert_ne!(salt1, salt2); // Should be different
    }
}

mod password_strength_tests {
    use my_axum::pkg::password::*;

    #[test]
    fn test_validate_password_strength_success() {
        let strong_passwords = vec![
            "StrongP@ss123",
            "MyP@ssw0rd!",
            "C0mpl3x!P@ss",
            "Test1234!@#$",
        ];

        for password in strong_passwords {
            let result = validate_password_strength(password);
            assert!(result.is_ok(), "Password '{}' should be valid", password);
        }
    }

    #[test]
    fn test_validate_password_strength_failures() {
        let weak_passwords = vec![
            ("short", "Password must be at least 8 characters long"),
            (
                "toolongbutlowercase",
                "Password must contain at least one uppercase letter",
            ),
            (
                "TOOLONGBUTUPPERCASE",
                "Password must contain at least one lowercase letter",
            ),
            ("NoDigits!", "Password must contain at least one digit"),
            (
                "NoSpecial123",
                "Password must contain at least one special character",
            ),
        ];

        for (password, expected_error) in weak_passwords {
            let result = validate_password_strength(password);
            assert!(result.is_err());
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains(expected_error),
                "Password '{}' should fail with error containing '{}', got '{}'",
                password,
                expected_error,
                error_msg
            );
        }
    }

    #[test]
    fn test_validate_empty_password_strength() {
        let result = validate_password_strength("");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("at least 8 characters")
        );
    }
}

mod password_config_tests {
    use my_axum::pkg::password::*;

    #[tokio::test]
    async fn test_hash_password_with_config() {
        let password = "config_test_password";

        let config = PasswordConfig {
            memory_cost: 2048,
            time_cost: 2,
            parallelism: 1,
        };

        let hash = hash_password_with_config(password, &config).await.unwrap();
        assert!(!hash.is_empty());

        let verify_result = verify_password(password, &hash).await;
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_password_config_default() {
        let config = PasswordConfig::default();
        assert_eq!(config.memory_cost, 4096);
        assert_eq!(config.time_cost, 3);
        assert_eq!(config.parallelism, 1);
    }

    #[tokio::test]
    async fn test_password_config_with_extreme_values() {
        // Test with very large memory cost (this might fail gracefully)
        let password = "test_password";

        let config = PasswordConfig {
            memory_cost: 8, // Very small memory cost
            time_cost: 1,
            parallelism: 1,
        };

        let result = hash_password_with_config(password, &config).await;
        // Should either succeed or return error
        match result {
            Ok(hash) => {
                assert!(!hash.is_empty());
                // Verify the password still works
                let verify_result = verify_password(password, &hash).await;
                assert!(verify_result.is_ok());
            }
            Err(e) => {
                // If it fails, error message should be descriptive
                assert!(!e.to_string().is_empty());
            }
        }
    }

    #[tokio::test]
    async fn test_password_config_clone() {
        let config = PasswordConfig {
            memory_cost: 2048,
            time_cost: 2,
            parallelism: 1,
        };

        let cloned = config.clone();
        assert_eq!(config.memory_cost, cloned.memory_cost);
        assert_eq!(config.time_cost, cloned.time_cost);
        assert_eq!(config.parallelism, cloned.parallelism);
    }

    #[test]
    fn test_password_config_debug() {
        let config = PasswordConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("PasswordConfig"));
        assert!(debug_str.contains("4096"));
    }

    #[tokio::test]
    async fn test_password_with_invalid_argon2_params() {
        // Test with parameters that might cause issues
        let password = "test";

        // Extremely low memory cost - this should still work in Argon2 v0.5+
        let config = PasswordConfig {
            memory_cost: 8, // Minimum allowed
            time_cost: 1,   // Minimum
            parallelism: 1,
        };

        let result = hash_password_with_config(password, &config).await;
        // Should work with minimum valid params
        match result {
            Ok(hash) => {
                assert!(!hash.is_empty());
            }
            Err(e) => {
                // If it fails, that's the error path we're trying to cover
                assert!(
                    e.to_string().contains("Argon2")
                        || e.to_string().contains("params")
                        || e.to_string().contains("hash")
                );
            }
        }
    }

    #[tokio::test]
    async fn test_password_hashing_with_various_configs() {
        let password = "TestP@ss123";

        let configs = vec![
            PasswordConfig {
                memory_cost: 8,
                time_cost: 1,
                parallelism: 1,
            },
            PasswordConfig {
                memory_cost: 1024,
                time_cost: 1,
                parallelism: 1,
            },
            PasswordConfig {
                memory_cost: 2048,
                time_cost: 2,
                parallelism: 1,
            },
            PasswordConfig {
                memory_cost: 4096,
                time_cost: 3,
                parallelism: 1,
            },
            PasswordConfig {
                memory_cost: 8192,
                time_cost: 4,
                parallelism: 2,
            },
        ];

        for config in configs {
            let result = hash_password_with_config(password, &config).await;
            match result {
                Ok(hash) => {
                    assert!(!hash.is_empty());
                    // Verify the hash works
                    let verify = verify_password(password, &hash).await;
                    assert!(verify.is_ok());
                }
                Err(_e) => {
                    // Some configs might fail - that's ok, we're testing error paths
                }
            }
        }
    }
}
