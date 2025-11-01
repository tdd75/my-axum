#[cfg(test)]
mod request_dto_tests {
    use axum::http::StatusCode;
    use my_axum::core::dto::util::deserialize_with_fields;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestRequestDTO {
        id: Option<u32>,
        name: Option<String>,
        email: Option<String>,
        active: Option<bool>,
        count: Option<i64>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct SimpleRequestDTO {
        message: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct RequiredFieldsDTO {
        id: u32,
        name: String,
        required_field: bool,
    }

    #[test]
    fn test_deserialize_with_fields_success() {
        let json_body = json!({
            "name": "Test User",
            "email": "test@example.com",
            "active": true
        });

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.name, Some("Test User".to_string()));
        assert_eq!(dto.email, Some("test@example.com".to_string()));
        assert_eq!(dto.active, Some(true));
        assert_eq!(dto.id, None);
        assert_eq!(dto.count, None);

        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"email".to_string()));
        assert!(fields.contains(&"active".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_partial_fields() {
        let json_body = json!({
            "id": 42,
            "email": "partial@example.com"
        });

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.id, Some(42));
        assert_eq!(dto.email, Some("partial@example.com".to_string()));
        assert_eq!(dto.name, None);
        assert_eq!(dto.active, None);
        assert_eq!(dto.count, None);

        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"id".to_string()));
        assert!(fields.contains(&"email".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_empty_object() {
        let json_body = json!({});

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.id, None);
        assert_eq!(dto.name, None);
        assert_eq!(dto.email, None);
        assert_eq!(dto.active, None);
        assert_eq!(dto.count, None);

        assert_eq!(fields.len(), 0);
    }

    #[test]
    fn test_deserialize_with_fields_simple_dto() {
        let json_body = json!({
            "message": "Hello World"
        });

        let result = deserialize_with_fields::<SimpleRequestDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.message, "Hello World");

        assert_eq!(fields.len(), 1);
        assert!(fields.contains(&"message".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_invalid_json_not_object() {
        let json_body = json!("not an object");

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Request body must be a JSON object");
    }

    #[test]
    fn test_deserialize_with_fields_invalid_json_array() {
        let json_body = json!([1, 2, 3]);

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Request body must be a JSON object");
    }

    #[test]
    fn test_deserialize_with_fields_invalid_json_null() {
        let json_body = json!(null);

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Request body must be a JSON object");
    }

    #[test]
    fn test_deserialize_with_fields_invalid_json_number() {
        let json_body = json!(123);

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Request body must be a JSON object");
    }

    #[test]
    fn test_deserialize_with_fields_invalid_json_boolean() {
        let json_body = json!(true);

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.message, "Request body must be a JSON object");
    }

    #[test]
    fn test_deserialize_with_fields_type_mismatch() {
        let json_body = json!({
            "id": "not_a_number",
            "name": "Valid Name"
        });

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert!(error.message.starts_with("Invalid request body:"));
    }

    #[test]
    fn test_deserialize_with_fields_missing_required_field() {
        let json_body = json!({
            "name": "Test User"
        });

        let result = deserialize_with_fields::<RequiredFieldsDTO>(json_body);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert!(error.message.starts_with("Invalid request body:"));
    }

    #[test]
    fn test_deserialize_with_fields_required_dto_success() {
        let json_body = json!({
            "id": 1,
            "name": "Required Test",
            "required_field": true
        });

        let result = deserialize_with_fields::<RequiredFieldsDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.id, 1);
        assert_eq!(dto.name, "Required Test");
        assert!(dto.required_field);

        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"id".to_string()));
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"required_field".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_extra_fields() {
        let json_body = json!({
            "name": "Test User",
            "extra_field": "should_be_ignored",
            "another_extra": 123
        });

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.name, Some("Test User".to_string()));

        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"extra_field".to_string()));
        assert!(fields.contains(&"another_extra".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_nested_object() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct NestedDTO {
            user: UserInfo,
            metadata: Option<String>,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct UserInfo {
            name: String,
            age: u8,
        }

        let json_body = json!({
            "user": {
                "name": "John Doe",
                "age": 30
            },
            "metadata": "some_metadata"
        });

        let result = deserialize_with_fields::<NestedDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.user.name, "John Doe");
        assert_eq!(dto.user.age, 30);
        assert_eq!(dto.metadata, Some("some_metadata".to_string()));

        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"user".to_string()));
        assert!(fields.contains(&"metadata".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_array_value() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct ArrayDTO {
            items: Vec<String>,
            count: Option<usize>,
        }

        let json_body = json!({
            "items": ["item1", "item2", "item3"],
            "count": 3
        });

        let result = deserialize_with_fields::<ArrayDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.items, vec!["item1", "item2", "item3"]);
        assert_eq!(dto.count, Some(3));

        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"items".to_string()));
        assert!(fields.contains(&"count".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_null_values() {
        let json_body = json!({
            "name": null,
            "email": "test@example.com",
            "active": null
        });

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.name, None);
        assert_eq!(dto.email, Some("test@example.com".to_string()));
        assert_eq!(dto.active, None);

        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"email".to_string()));
        assert!(fields.contains(&"active".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_field_order_preservation() {
        let json_body = json!({
            "z_field": "last",
            "a_field": "first",
            "m_field": "middle"
        });

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct OrderTestDTO {
            a_field: Option<String>,
            m_field: Option<String>,
            z_field: Option<String>,
        }

        let result = deserialize_with_fields::<OrderTestDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.a_field, Some("first".to_string()));
        assert_eq!(dto.m_field, Some("middle".to_string()));
        assert_eq!(dto.z_field, Some("last".to_string()));

        // Fields should be extracted (order may vary depending on HashMap implementation)
        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"a_field".to_string()));
        assert!(fields.contains(&"m_field".to_string()));
        assert!(fields.contains(&"z_field".to_string()));
    }

    #[test]
    fn test_deserialize_with_fields_empty_string_values() {
        let json_body = json!({
            "name": "",
            "email": "",
            "id": 0
        });

        let result = deserialize_with_fields::<TestRequestDTO>(json_body);
        assert!(result.is_ok());

        let (dto, fields) = result.unwrap();
        assert_eq!(dto.name, Some("".to_string()));
        assert_eq!(dto.email, Some("".to_string()));
        assert_eq!(dto.id, Some(0));

        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"email".to_string()));
        assert!(fields.contains(&"id".to_string()));
    }
}
