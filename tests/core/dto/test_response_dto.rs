#[cfg(test)]
mod response_dto_tests {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use my_axum::core::dto::response_dto::ResponseDTO;
    use serde::Serialize;

    #[derive(Debug, Serialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
        active: bool,
    }

    #[derive(Debug, Serialize, PartialEq)]
    struct SimpleData {
        message: String,
    }

    #[test]
    fn test_response_dto_new() {
        let data = TestData {
            id: 1,
            name: "Test".to_string(),
            active: true,
        };

        let response = ResponseDTO::new(StatusCode::OK, data);

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.data.id, 1);
        assert_eq!(response.data.name, "Test");
        assert!(response.data.active);
    }

    #[test]
    fn test_response_dto_new_with_different_status_codes() {
        let data = SimpleData {
            message: "Created successfully".to_string(),
        };

        let response = ResponseDTO::new(StatusCode::CREATED, data);

        assert_eq!(response.status, StatusCode::CREATED);
        assert_eq!(response.data.message, "Created successfully");
    }

    #[test]
    fn test_response_dto_new_with_empty_struct() {
        #[derive(Debug, Serialize)]
        struct EmptyData {}

        let data = EmptyData {};
        let response = ResponseDTO::new(StatusCode::NO_CONTENT, data);

        assert_eq!(response.status, StatusCode::NO_CONTENT);
    }

    #[test]
    fn test_response_dto_new_with_vec() {
        let data = vec![
            TestData {
                id: 1,
                name: "First".to_string(),
                active: true,
            },
            TestData {
                id: 2,
                name: "Second".to_string(),
                active: false,
            },
        ];

        let response = ResponseDTO::new(StatusCode::OK, data);

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].name, "First");
        assert_eq!(response.data[1].name, "Second");
    }

    #[test]
    fn test_response_dto_new_with_option() {
        let data: Option<TestData> = Some(TestData {
            id: 42,
            name: "Optional".to_string(),
            active: true,
        });

        let response = ResponseDTO::new(StatusCode::OK, data);

        assert_eq!(response.status, StatusCode::OK);
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().id, 42);
    }

    #[test]
    fn test_response_dto_new_with_none() {
        let data: Option<TestData> = None;
        let response = ResponseDTO::new(StatusCode::NOT_FOUND, data);

        assert_eq!(response.status, StatusCode::NOT_FOUND);
        assert!(response.data.is_none());
    }

    #[test]
    fn test_response_dto_into_response() {
        let data = SimpleData {
            message: "Hello, World!".to_string(),
        };

        let dto = ResponseDTO::new(StatusCode::OK, data);
        let response = dto.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_response_dto_into_response_with_different_status() {
        let data = SimpleData {
            message: "Resource created".to_string(),
        };

        let dto = ResponseDTO::new(StatusCode::CREATED, data);
        let response = dto.into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[test]
    fn test_response_dto_into_response_client_error() {
        let data = SimpleData {
            message: "Bad request".to_string(),
        };

        let dto = ResponseDTO::new(StatusCode::BAD_REQUEST, data);
        let response = dto.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_response_dto_into_response_server_error() {
        let data = SimpleData {
            message: "Internal server error".to_string(),
        };

        let dto = ResponseDTO::new(StatusCode::INTERNAL_SERVER_ERROR, data);
        let response = dto.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_response_dto_with_string_data() {
        let data = "Simple string response".to_string();
        let response = ResponseDTO::new(StatusCode::OK, data);

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.data, "Simple string response");
    }

    #[test]
    fn test_response_dto_with_number_data() {
        let data = 12345u64;
        let response = ResponseDTO::new(StatusCode::OK, data);

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.data, 12345);
    }

    #[test]
    fn test_response_dto_with_bool_data() {
        let data = true;
        let response = ResponseDTO::new(StatusCode::OK, data);

        assert_eq!(response.status, StatusCode::OK);
        assert!(response.data);
    }

    #[test]
    fn test_response_dto_debug_trait() {
        let data = SimpleData {
            message: "Debug test".to_string(),
        };

        let response = ResponseDTO::new(StatusCode::OK, data);
        let debug_string = format!("{:?}", response);

        // Debug should contain both status and data information
        assert!(debug_string.contains("200") || debug_string.contains("OK"));
        assert!(debug_string.contains("Debug test"));
    }

    #[test]
    fn test_response_dto_with_nested_struct() {
        #[derive(Debug, Serialize)]
        struct NestedData {
            user: TestData,
            metadata: SimpleData,
        }

        let data = NestedData {
            user: TestData {
                id: 1,
                name: "User".to_string(),
                active: true,
            },
            metadata: SimpleData {
                message: "Some metadata".to_string(),
            },
        };

        let response = ResponseDTO::new(StatusCode::OK, data);

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.data.user.id, 1);
        assert_eq!(response.data.metadata.message, "Some metadata");
    }

    #[test]
    fn test_response_dto_multiple_instances() {
        let response1 = ResponseDTO::new(StatusCode::OK, "First response".to_string());
        let response2 = ResponseDTO::new(StatusCode::CREATED, "Second response".to_string());
        let response3 = ResponseDTO::new(StatusCode::ACCEPTED, "Third response".to_string());

        assert_eq!(response1.status, StatusCode::OK);
        assert_eq!(response2.status, StatusCode::CREATED);
        assert_eq!(response3.status, StatusCode::ACCEPTED);

        assert_eq!(response1.data, "First response");
        assert_eq!(response2.data, "Second response");
        assert_eq!(response3.data, "Third response");
    }

    #[test]
    fn test_response_dto_with_result_ok() {
        let data: Result<TestData, String> = Ok(TestData {
            id: 1,
            name: "Success".to_string(),
            active: true,
        });

        let response = ResponseDTO::new(StatusCode::OK, data);

        assert_eq!(response.status, StatusCode::OK);
        assert!(response.data.is_ok());
        assert_eq!(response.data.unwrap().name, "Success");
    }

    #[test]
    fn test_response_dto_with_result_err() {
        let data: Result<TestData, String> = Err("Something went wrong".to_string());
        let response = ResponseDTO::new(StatusCode::BAD_REQUEST, data);

        assert_eq!(response.status, StatusCode::BAD_REQUEST);
        assert!(response.data.is_err());
        assert_eq!(response.data.unwrap_err(), "Something went wrong");
    }
}
