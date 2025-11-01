#[cfg(test)]
mod util_tests {
    use axum::http::StatusCode;
    use my_axum::core::dto::error_dto::ErrorDTO;
    use my_axum::core::dto::util::{ToJson, deserialize_with_fields, serialize_status_code};
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct TestDto {
        name: String,
        age: i32,
        email: Option<String>,
    }

    #[test]
    fn test_serialize_status_code() {
        #[derive(Serialize)]
        struct TestStruct {
            #[serde(serialize_with = "serialize_status_code")]
            status: StatusCode,
        }

        let test = TestStruct {
            status: StatusCode::OK,
        };
        let serialized = serde_json::to_string(&test).unwrap();
        assert!(serialized.contains("\"status\":200"));

        let test = TestStruct {
            status: StatusCode::NOT_FOUND,
        };
        let serialized = serde_json::to_string(&test).unwrap();
        assert!(serialized.contains("\"status\":404"));
    }

    #[test]
    fn test_deserialize_with_fields_success() {
        let body = json!({
            "name": "John Doe",
            "age": 30,
            "email": "john@example.com"
        });

        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_ok());
        let (dto, fields) = result.unwrap();
        assert_eq!(dto.name, "John Doe");
        assert_eq!(dto.age, 30);
        assert_eq!(dto.email, Some("john@example.com".to_string()));
        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"age".to_string()));
        assert!(fields.contains(&"email".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_partial() {
        let body = json!({
            "name": "Jane",
            "age": 25
        });

        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_ok());
        let (dto, fields) = result.unwrap();
        assert_eq!(dto.name, "Jane");
        assert_eq!(dto.age, 25);
        assert_eq!(dto.email, None);
        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"age".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_invalid_json_not_object() {
        let body = json!("not an object");

        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Request body must be a JSON object");
    }

    #[test]
    fn test_deserialize_with_fields_invalid_json_array() {
        let body = json!(["not", "an", "object"]);

        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_deserialize_with_fields_invalid_json_null() {
        let body = Value::Null;

        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_deserialize_with_fields_type_mismatch() {
        let body = json!({
            "name": "John",
            "age": "not a number",
            "email": "john@example.com"
        });

        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert!(error.message.contains("Invalid request body"));
    }

    #[test]
    fn test_deserialize_with_fields_missing_required() {
        let body = json!({
            "email": "test@example.com"
        });

        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_deserialize_with_fields_empty_object() {
        let body = json!({});

        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_with_fields_extra_fields() {
        let body = json!({
            "name": "John",
            "age": 30,
            "email": "john@example.com",
            "extra_field": "should be ignored"
        });

        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_ok());
        let (dto, fields) = result.unwrap();
        assert_eq!(dto.name, "John");
        assert_eq!(fields.len(), 4);
        assert!(fields.contains(&"extra_field".to_string()));
    }

    #[test]
    fn test_to_json_trait_for_error_dto() {
        let error = ErrorDTO::new(StatusCode::BAD_REQUEST, "Test".to_string());
        let json_value = error.to_json();

        assert!(json_value.is_object());
        assert_eq!(json_value["status"], 400);
        assert_eq!(json_value["message"], "Test");
    }

    #[test]
    fn test_to_json_string_trait_for_error_dto() {
        let error = ErrorDTO::new(StatusCode::NOT_FOUND, "Not found".to_string());
        let json_string = error.to_json_string();

        assert!(json_string.contains("404"));
        assert!(json_string.contains("Not found"));
    }

    #[derive(Serialize)]
    struct CustomStruct {
        field1: String,
        field2: i32,
    }

    impl ToJson for CustomStruct {}

    #[test]
    fn test_to_json_trait_custom_struct() {
        let custom = CustomStruct {
            field1: "test".to_string(),
            field2: 42,
        };

        let json_value = custom.to_json();
        assert!(json_value.is_object());
        assert_eq!(json_value["field1"], "test");
        assert_eq!(json_value["field2"], 42);
    }

    #[test]
    fn test_to_json_string_trait_custom_struct() {
        let custom = CustomStruct {
            field1: "hello".to_string(),
            field2: 100,
        };

        let json_string = custom.to_json_string();
        assert!(json_string.contains("\"field1\":\"hello\""));
        assert!(json_string.contains("\"field2\":100"));
    }

    #[test]
    fn test_deserialize_with_fields_with_nested_object() {
        #[derive(Deserialize)]
        struct NestedDto {
            name: String,
            details: Value,
        }

        let body = json!({
            "name": "Test",
            "details": {
                "age": 30,
                "city": "NYC"
            }
        });

        let result: Result<(NestedDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_ok());
        let (dto, fields) = result.unwrap();
        assert_eq!(dto.name, "Test");
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn test_serialize_status_code_all_common_codes() {
        #[derive(Serialize)]
        struct TestStruct {
            #[serde(serialize_with = "serialize_status_code")]
            status: StatusCode,
        }

        let test_codes = vec![
            (StatusCode::OK, 200),
            (StatusCode::CREATED, 201),
            (StatusCode::BAD_REQUEST, 400),
            (StatusCode::UNAUTHORIZED, 401),
            (StatusCode::FORBIDDEN, 403),
            (StatusCode::NOT_FOUND, 404),
            (StatusCode::INTERNAL_SERVER_ERROR, 500),
        ];

        for (status_code, expected_num) in test_codes {
            let test = TestStruct {
                status: status_code,
            };
            let serialized = serde_json::to_string(&test).unwrap();
            assert!(serialized.contains(&format!("\"status\":{}", expected_num)));
        }
    }

    #[test]
    fn test_deserialize_with_fields_field_order_preserved() {
        let body = json!({
            "name": "Test",
            "age": 25,
            "email": "test@example.com"
        });

        let result: Result<(TestDto, Vec<String>), ErrorDTO> = deserialize_with_fields(body);

        assert!(result.is_ok());
        let (_, fields) = result.unwrap();

        // Verify all fields are present (order may vary in HashMap)
        assert_eq!(fields.len(), 3);
        let field_set: std::collections::HashSet<_> = fields.into_iter().collect();
        assert!(field_set.contains("name"));
        assert!(field_set.contains("age"));
        assert!(field_set.contains("email"));
    }
}
