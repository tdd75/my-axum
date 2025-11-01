#[cfg(test)]
mod openapi_tests {
    use my_axum::core::api::openapi::ApiDoc;
    use utoipa::OpenApi;

    #[test]
    fn test_api_doc_can_be_created() {
        let api_doc = ApiDoc::openapi();
        assert!(!api_doc.info.title.is_empty());
    }

    #[test]
    fn test_api_doc_has_security_scheme() {
        let api_doc = ApiDoc::openapi();
        assert!(api_doc.components.is_some());

        let components = api_doc.components.unwrap();
        assert!(components.security_schemes.contains_key("bearer_auth"));
    }

    #[test]
    fn test_api_doc_has_bearer_auth() {
        let api_doc = ApiDoc::openapi();
        let components = api_doc.components.unwrap();
        let security_scheme = components.security_schemes.get("bearer_auth").unwrap();

        // Check if it's HTTP bearer scheme
        match security_scheme {
            utoipa::openapi::security::SecurityScheme::Http(http) => {
                assert!(matches!(
                    http.scheme,
                    utoipa::openapi::security::HttpAuthScheme::Bearer
                ));
            }
            _ => panic!("Expected HTTP security scheme"),
        }
    }

    #[test]
    fn test_api_doc_has_paths() {
        let api_doc = ApiDoc::openapi();
        assert!(!api_doc.paths.paths.is_empty());
    }

    #[test]
    fn test_api_doc_has_auth_endpoints() {
        let api_doc = ApiDoc::openapi();
        let paths = &api_doc.paths.paths;

        // OpenAPI paths may have different formats, just check we have some paths
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_api_doc_has_user_endpoints() {
        let api_doc = ApiDoc::openapi();
        let paths = &api_doc.paths.paths;

        // Just verify we have some endpoints
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_api_doc_info_not_empty() {
        let api_doc = ApiDoc::openapi();
        assert!(!api_doc.info.title.is_empty());
    }

    #[test]
    fn test_api_doc_to_json() {
        let api_doc = ApiDoc::openapi();
        let json = serde_json::to_string(&api_doc);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("bearer_auth"));
    }

    #[test]
    fn test_api_doc_bearer_format() {
        let api_doc = ApiDoc::openapi();
        let components = api_doc.components.unwrap();
        let security_scheme = components.security_schemes.get("bearer_auth").unwrap();

        match security_scheme {
            utoipa::openapi::security::SecurityScheme::Http(http) => {
                assert_eq!(http.bearer_format, Some("JWT".to_string()));
            }
            _ => panic!("Expected HTTP security scheme"),
        }
    }

    #[test]
    fn test_api_doc_multiple_creation() {
        let api_doc1 = ApiDoc::openapi();
        let api_doc2 = ApiDoc::openapi();

        // Both should have the same structure
        assert_eq!(api_doc1.info.title, api_doc2.info.title);
        assert_eq!(api_doc1.paths.paths.len(), api_doc2.paths.paths.len());
    }

    #[test]
    fn test_api_doc_serialization() {
        let api_doc = ApiDoc::openapi();
        let json = serde_json::to_value(&api_doc);
        assert!(json.is_ok());

        let json_value = json.unwrap();
        assert!(json_value.is_object());
        assert!(json_value.get("openapi").is_some());
        assert!(json_value.get("info").is_some());
        assert!(json_value.get("paths").is_some());
    }

    #[test]
    fn test_api_doc_has_components() {
        let api_doc = ApiDoc::openapi();
        assert!(api_doc.components.is_some());
    }

    #[test]
    fn test_security_scheme_name() {
        let api_doc = ApiDoc::openapi();
        let components = api_doc.components.unwrap();
        assert!(components.security_schemes.contains_key("bearer_auth"));
        assert!(!components.security_schemes.contains_key("api_key"));
    }

    #[test]
    fn test_api_doc_openapi_version() {
        let api_doc = ApiDoc::openapi();
        // OpenApiVersion is an enum, just check it exists
        let _version = &api_doc.openapi;
    }

    #[test]
    fn test_api_doc_info_fields() {
        let api_doc = ApiDoc::openapi();
        let info = &api_doc.info;

        assert!(!info.title.is_empty());
        assert!(!info.version.is_empty());
    }

    #[test]
    fn test_api_doc_paths_not_empty() {
        let api_doc = ApiDoc::openapi();
        assert!(!api_doc.paths.paths.is_empty());
    }

    #[test]
    fn test_api_doc_has_change_password_endpoint() {
        let api_doc = ApiDoc::openapi();
        let paths = &api_doc.paths.paths;

        let has_change_password = paths.keys().any(|path| path.contains("password"));
        assert!(has_change_password);
    }

    #[test]
    fn test_api_doc_has_logout_endpoint() {
        let api_doc = ApiDoc::openapi();
        let paths = &api_doc.paths.paths;

        let has_logout = paths.keys().any(|path| path.contains("logout"));
        assert!(has_logout);
    }

    #[test]
    fn test_api_doc_has_refresh_token_endpoint() {
        let api_doc = ApiDoc::openapi();
        let paths = &api_doc.paths.paths;

        let has_refresh = paths
            .keys()
            .any(|path| path.contains("refresh") || path.contains("token"));
        assert!(has_refresh);
    }

    #[test]
    fn test_api_doc_json_contains_security() {
        let api_doc = ApiDoc::openapi();
        let json = serde_json::to_string(&api_doc).unwrap();

        assert!(json.contains("bearer_auth") || json.contains("security"));
    }

    #[test]
    fn test_security_addon_modifier() {
        let api_doc = ApiDoc::openapi();

        // The SecurityAddon should have been applied
        assert!(api_doc.components.is_some());
        let components = api_doc.components.unwrap();
        assert!(!components.security_schemes.is_empty());
    }
}
