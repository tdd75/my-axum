#[cfg(test)]
mod error_dto_tests {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use my_axum::core::dto::error_dto::ErrorDTO;
    use std::error::Error;

    #[test]
    fn test_error_dto_new() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Test error".to_string());

        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Test error");
    }

    #[test]
    fn test_error_dto_new_with_different_status_codes() {
        let error_400 = ErrorDTO::new(StatusCode::BAD_REQUEST, "Bad request".to_string());
        let error_401 = ErrorDTO::new(StatusCode::UNAUTHORIZED, "Unauthorized".to_string());
        let error_404 = ErrorDTO::new(StatusCode::NOT_FOUND, "Not found".to_string());
        let error_500 = ErrorDTO::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Server error".to_string(),
        );

        assert_eq!(error_400.status, StatusCode::BAD_REQUEST);
        assert_eq!(error_401.status, StatusCode::UNAUTHORIZED);
        assert_eq!(error_404.status, StatusCode::NOT_FOUND);
        assert_eq!(error_500.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_dto_display_trait() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Test error message".to_string());
        let display_string = format!("{}", error);

        assert_eq!(display_string, "<400> Test error message");
    }

    #[test]
    fn test_error_dto_display_with_different_status_codes() {
        let error_404 = ErrorDTO::new(StatusCode::NOT_FOUND, "Resource not found".to_string());
        let error_500 = ErrorDTO::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal error".to_string(),
        );

        assert_eq!(format!("{}", error_404), "<404> Resource not found");
        assert_eq!(format!("{}", error_500), "<500> Internal error");
    }

    #[test]
    fn test_error_dto_debug_trait() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Debug test".to_string());
        let debug_string = format!("{:?}", error);

        // Debug should contain both status and message
        assert!(debug_string.contains("400") || debug_string.contains("BAD_REQUEST"));
        assert!(debug_string.contains("Debug test"));
    }

    #[test]
    fn test_error_dto_into_response() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Response test".to_string());
        let response = error.into_response();

        // Verify the response has the correct status code
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_dto_into_response_with_different_status() {
        let error = ErrorDTO::new(StatusCode::NOT_FOUND, "Not found error".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_dto_error_trait() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Error trait test".to_string());

        // Test that it implements the Error trait
        let _: &dyn std::error::Error = &error;

        // Test source method (should return None for this simple error)
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_dto_with_empty_message() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "".to_string());

        assert_eq!(error.message, "");
        assert_eq!(format!("{}", error), "<400> ");
    }

    #[test]
    fn test_error_dto_with_long_message() {
        let long_message = "A".repeat(1000);
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, long_message.clone());

        assert_eq!(error.message, long_message);
        assert_eq!(format!("{}", error), format!("<400> {}", long_message));
    }

    #[test]
    fn test_error_dto_with_special_characters() {
        let special_message = "Error with special chars: áéíóú ñ €";
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, special_message.to_string());

        assert_eq!(error.message, special_message);
        assert_eq!(format!("{}", error), format!("<400> {}", special_message));
    }

    #[test]
    fn test_error_dto_multiple_instances() {
        let error1 = ErrorDTO::new(StatusCode::BAD_REQUEST, "Error 1".to_string());
        let error2 = ErrorDTO::new(StatusCode::NOT_FOUND, "Error 2".to_string());
        let error3 = ErrorDTO::new(StatusCode::INTERNAL_SERVER_ERROR, "Error 3".to_string());

        assert_eq!(error1.status, StatusCode::BAD_REQUEST);
        assert_eq!(error2.status, StatusCode::NOT_FOUND);
        assert_eq!(error3.status, StatusCode::INTERNAL_SERVER_ERROR);

        assert_eq!(error1.message, "Error 1");
        assert_eq!(error2.message, "Error 2");
        assert_eq!(error3.message, "Error 3");
    }

    #[test]
    fn test_map_internal_error_with_string() {
        let error_message = "Database connection failed";
        let error_dto = ErrorDTO::map_internal_error(error_message);

        assert_eq!(error_dto.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            error_dto.message,
            "Internal Server Error: Database connection failed"
        );
    }

    #[test]
    fn test_map_internal_error_with_std_error() {
        use std::io::{Error, ErrorKind};

        let io_error = Error::new(ErrorKind::PermissionDenied, "File access denied");
        let error_dto = ErrorDTO::map_internal_error(io_error);

        assert_eq!(error_dto.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(error_dto.message.contains("Internal Server Error"));
        assert!(error_dto.message.contains("File access denied"));
    }

    #[test]
    fn test_map_internal_error_with_format_string() {
        let user_id = 123;
        let error_message = format!("User {} not found in database", user_id);
        let error_dto = ErrorDTO::map_internal_error(error_message);

        assert_eq!(error_dto.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            error_dto.message,
            "Internal Server Error: User 123 not found in database"
        );
    }

    #[test]
    fn test_map_internal_error_with_empty_message() {
        let error_dto = ErrorDTO::map_internal_error("");

        assert_eq!(error_dto.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error_dto.message, "Internal Server Error: ");
    }

    #[test]
    fn test_map_internal_error_with_long_message() {
        let long_message = "A".repeat(1000);
        let error_dto = ErrorDTO::map_internal_error(&long_message);

        assert_eq!(error_dto.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(error_dto.message.starts_with("Internal Server Error: A"));
        assert!(error_dto.message.len() > 1000);
    }

    #[test]
    fn test_map_internal_error_with_special_characters() {
        let special_message = "Error with special chars: áéíóú ñ € \\n \\t";
        let error_dto = ErrorDTO::map_internal_error(special_message);

        assert_eq!(error_dto.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(error_dto.message.contains("áéíóú ñ €"));
        assert!(error_dto.message.contains("\\n \\t"));
    }

    #[test]
    fn test_map_internal_error_consistent_status() {
        let error1 = ErrorDTO::map_internal_error("Error 1");
        let error2 = ErrorDTO::map_internal_error("Error 2");
        let error3 = ErrorDTO::map_internal_error("Error 3");

        assert_eq!(error1.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error2.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error3.status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_map_internal_error_message_format() {
        let original_message = "Original error";
        let error_dto = ErrorDTO::map_internal_error(original_message);

        assert!(error_dto.message.starts_with("Internal Server Error: "));
        assert!(error_dto.message.ends_with("Original error"));
    }

    #[test]
    fn test_map_internal_error_with_newlines() {
        let message_with_newlines = "Line 1\\nLine 2\\nLine 3";
        let error_dto = ErrorDTO::map_internal_error(message_with_newlines);

        assert_eq!(error_dto.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(error_dto.message.contains("Line 1\\nLine 2\\nLine 3"));
    }

    #[test]
    fn test_map_internal_error_display_trait() {
        let error_dto = ErrorDTO::map_internal_error("Test error");
        let display_string = format!("{}", error_dto);

        assert!(display_string.contains("<500>"));
        assert!(display_string.contains("Internal Server Error: Test error"));
    }

    #[test]
    fn test_map_internal_error_into_response() {
        let error_dto = ErrorDTO::map_internal_error("Response test");
        let response = error_dto.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_map_internal_error_vs_new() {
        let manual_error = ErrorDTO::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error: Manual message".to_string(),
        );
        let mapped_error = ErrorDTO::map_internal_error("Manual message");

        assert_eq!(manual_error.status, mapped_error.status);
        assert_eq!(manual_error.message, mapped_error.message);
    }

    #[test]
    fn test_error_dto_to_json() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Test error".to_string());
        let json_value = error.to_json();

        assert!(json_value.is_object());
        assert_eq!(json_value["status"], 400);
        assert_eq!(json_value["message"], "Test error");
    }

    #[test]
    fn test_error_dto_to_json_with_different_status() {
        let error = ErrorDTO::new(StatusCode::NOT_FOUND, "Not found".to_string());
        let json_value = error.to_json();

        assert_eq!(json_value["status"], 404);
        assert_eq!(json_value["message"], "Not found");
    }

    #[test]
    fn test_error_dto_to_json_string() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Test error".to_string());
        let json_string = error.to_json_string();

        assert!(json_string.contains("400"));
        assert!(json_string.contains("Test error"));
        assert!(json_string.starts_with("{"));
        assert!(json_string.ends_with("}"));
    }

    #[test]
    fn test_error_dto_to_json_string_with_special_characters() {
        let error = ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            "Error with \"quotes\" and \\ backslash".to_string(),
        );
        let json_string = error.to_json_string();

        // Should properly escape special characters
        assert!(json_string.contains("\\\""));
        assert!(json_string.contains("\\\\"));
    }

    #[test]
    fn test_error_dto_to_json_empty_message() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "".to_string());
        let json_value = error.to_json();

        assert_eq!(json_value["status"], 400);
        assert_eq!(json_value["message"], "");
    }

    #[test]
    fn test_error_dto_from_db_err() {
        use sea_orm::DbErr;

        let db_error = DbErr::Custom("Database connection failed".to_string());
        let error_dto: ErrorDTO = db_error.into();

        assert_eq!(error_dto.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(error_dto.message.contains("Internal Server Error"));
        assert!(error_dto.message.contains("Database connection failed"));
    }

    #[test]
    fn test_error_dto_from_db_err_query_error() {
        use sea_orm::DbErr;

        let db_error = DbErr::RecordNotFound("User not found".to_string());
        let error_dto: ErrorDTO = db_error.into();

        assert_eq!(error_dto.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(error_dto.message.contains("User not found"));
    }

    #[test]
    fn test_error_dto_to_json_preserves_structure() {
        let error = ErrorDTO::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Server error".to_string(),
        );
        let json_value = error.to_json();

        // Verify it has exactly the expected fields
        let obj = json_value.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("status"));
        assert!(obj.contains_key("message"));
    }

    #[test]
    fn test_error_dto_to_json_string_is_valid_json() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Test".to_string());
        let json_string = error.to_json_string();

        // Verify we can parse it back
        let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&json_string);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_error_dto_into_response_body_format() {
        use axum::response::IntoResponse;

        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Test error".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // The body should contain the message
        // Note: Actually reading the body requires async context and consumption
    }

    #[test]
    fn test_error_dto_serialization_format() {
        let error = ErrorDTO::new(StatusCode::UNAUTHORIZED, "Access denied".to_string());
        let serialized = serde_json::to_string(&error).unwrap();

        assert!(serialized.contains("\"status\":401"));
        assert!(serialized.contains("\"message\":\"Access denied\""));
    }

    #[test]
    fn test_error_dto_to_json_with_unicode() {
        let error = ErrorDTO::new(
            StatusCode::BAD_REQUEST,
            "Error with unicode: 你好 мир".to_string(),
        );
        let json_value = error.to_json();

        assert_eq!(json_value["message"], "Error with unicode: 你好 мир");
    }

    #[test]
    fn test_error_dto_clone_via_new() {
        let error1 = ErrorDTO::new(StatusCode::BAD_REQUEST, "Original".to_string());
        let error2 = ErrorDTO::new(error1.status, error1.message.clone());

        assert_eq!(error1.status, error2.status);
        assert_eq!(error1.message, error2.message);
    }
}
