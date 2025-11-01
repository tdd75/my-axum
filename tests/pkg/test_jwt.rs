mod encode_decode_tests {
    use chrono::Duration;
    use my_axum::pkg::jwt::*;

    #[test]
    fn test_encode_decode_token_success() {
        let secret = "test-secret-key";
        let user_id = 123;
        let expires_delta = Duration::minutes(30);

        // Encode token
        let token = encode_token(user_id, expires_delta, secret).unwrap();
        assert!(!token.is_empty());

        // Decode token
        let claims = decode_token(&token, secret).unwrap();
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_claims_structure() {
        let secret = "test-secret-key";
        let user_id = 456;
        let expires_delta = Duration::hours(1);

        let token = encode_token(user_id, expires_delta, secret).unwrap();
        let claims = decode_token(&token, secret).unwrap();

        assert_eq!(claims.sub, user_id);
        assert!(claims.iat > 0);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_multiple_users() {
        let secret = "test-secret-key";
        let expires_delta = Duration::minutes(15);

        let users = vec![1, 100, 999, 1234567];

        for user_id in users {
            let token = encode_token(user_id, expires_delta, secret).unwrap();
            let claims = decode_token(&token, secret).unwrap();
            assert_eq!(claims.sub, user_id);
        }
    }
}

mod token_validation_tests {
    use chrono::Duration;
    use my_axum::pkg::jwt::*;

    #[test]
    fn test_decode_token_with_wrong_secret() {
        let secret = "test-secret-key";
        let wrong_secret = "wrong-secret-key";
        let user_id = 123;
        let expires_delta = Duration::minutes(30);

        // Encode with correct secret
        let token = encode_token(user_id, expires_delta, secret).unwrap();

        // Try to decode with wrong secret
        let result = decode_token(&token, wrong_secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_expiration() {
        let secret = "test-secret-key";
        let user_id = 123;
        let expires_delta = Duration::seconds(-3600); // Expired 1 hour ago

        // Encode expired token
        let token = encode_token(user_id, expires_delta, secret).unwrap();

        // Try to decode expired token
        let result = decode_token(&token, secret);
        assert!(result.is_err(), "Token should be expired and invalid");
    }

    #[test]
    fn test_invalid_token_format() {
        let secret = "test-secret-key";
        let invalid_tokens = vec![
            "",
            "invalid.token",
            "header.payload", // Missing signature
            "not.a.jwt.token.at.all",
        ];

        for token in invalid_tokens {
            let result = decode_token(token, secret);
            assert!(result.is_err());
        }
    }
}

mod security_tests {
    use chrono::Duration;
    use my_axum::pkg::jwt::*;

    #[test]
    fn test_different_secrets_produce_different_tokens() {
        let secret1 = "secret-one";
        let secret2 = "secret-two";
        let user_id = 123;
        let expires_delta = Duration::minutes(30);

        let token1 = encode_token(user_id, expires_delta, secret1).unwrap();
        let token2 = encode_token(user_id, expires_delta, secret2).unwrap();

        assert_ne!(token1, token2);

        // Each should only decode with its own secret
        assert!(decode_token(&token1, secret1).is_ok());
        assert!(decode_token(&token2, secret2).is_ok());
        assert!(decode_token(&token1, secret2).is_err());
        assert!(decode_token(&token2, secret1).is_err());
    }

    #[test]
    fn test_long_secret() {
        let long_secret = "a".repeat(1000);
        let user_id = 123;
        let expires_delta = Duration::minutes(30);

        let token = encode_token(user_id, expires_delta, &long_secret).unwrap();
        let claims = decode_token(&token, &long_secret).unwrap();
        assert_eq!(claims.sub, user_id);
    }
}
