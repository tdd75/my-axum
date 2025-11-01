use my_axum::pkg::url::*;

#[test]
fn test_url_builder_basic() {
    let url = UrlBuilder::new("example.com").build();
    assert_eq!(url, "example.com");
}

#[test]
fn test_url_builder_with_scheme() {
    let url = UrlBuilder::new("example.com").scheme("https").build();
    assert_eq!(url, "https://example.com");
}

#[test]
fn test_url_builder_with_port() {
    let url = UrlBuilder::new("example.com").port(8080).build();
    assert_eq!(url, "example.com:8080");
}

#[test]
fn test_url_builder_with_credentials() {
    let url = UrlBuilder::new("example.com")
        .username("user")
        .password("pass")
        .build();
    assert_eq!(url, "user:pass@example.com");
}

#[test]
fn test_url_builder_with_username_only() {
    let url = UrlBuilder::new("example.com").username("user").build();
    assert_eq!(url, "user@example.com");
}

#[test]
fn test_url_builder_complete_url() {
    let url = UrlBuilder::new("example.com")
        .scheme("https")
        .username("user")
        .password("pass")
        .port(8443)
        .path("/api/v1/users")
        .query("limit=10&offset=0")
        .fragment("section1")
        .build();

    assert_eq!(
        url,
        "https://user:pass@example.com:8443/api/v1/users?limit=10&offset=0#section1"
    );
}

#[test]
fn test_url_builder_path_auto_slash() {
    let url = UrlBuilder::new("example.com").path("api/v1/users").build();
    assert_eq!(url, "example.com/api/v1/users");
}

#[test]
fn test_url_builder_path_with_slash() {
    let url = UrlBuilder::new("example.com").path("/api/v1/users").build();
    assert_eq!(url, "example.com/api/v1/users");
}

#[test]
fn test_url_builder_display_trait() {
    let builder = UrlBuilder::new("example.com").scheme("https").port(443);

    let url_string = format!("{}", builder);
    assert_eq!(url_string, "https://example.com:443");
}

#[test]
fn test_url_builder_clone() {
    let builder1 = UrlBuilder::new("example.com").scheme("https").port(443);

    let builder2 = builder1.clone();
    assert_eq!(builder1.build(), builder2.build());
}

#[test]
fn test_parse_url_basic() {
    let components = parse_url("example.com").unwrap();
    assert_eq!(components.host, "example.com");
    assert_eq!(components.scheme, None);
    assert_eq!(components.port, None);
}

#[test]
fn test_parse_url_with_scheme() {
    let components = parse_url("https://example.com").unwrap();
    assert_eq!(components.scheme, Some("https".to_string()));
    assert_eq!(components.host, "example.com");
}

#[test]
fn test_parse_url_with_port() {
    let components = parse_url("https://example.com:8080").unwrap();
    assert_eq!(components.scheme, Some("https".to_string()));
    assert_eq!(components.host, "example.com");
    assert_eq!(components.port, Some(8080));
}

#[test]
fn test_parse_url_with_credentials() {
    let components = parse_url("https://user:pass@example.com:8080").unwrap();
    assert_eq!(components.scheme, Some("https".to_string()));
    assert_eq!(components.username, Some("user".to_string()));
    assert_eq!(components.password, Some("pass".to_string()));
    assert_eq!(components.host, "example.com");
    assert_eq!(components.port, Some(8080));
}

#[test]
fn test_parse_url_with_path() {
    let components = parse_url("https://example.com/api/v1/users").unwrap();
    assert_eq!(components.scheme, Some("https".to_string()));
    assert_eq!(components.host, "example.com");
    assert_eq!(components.path, Some("/api/v1/users".to_string()));
}

#[test]
fn test_parse_url_with_query() {
    let components = parse_url("https://example.com/api?limit=10").unwrap();
    assert_eq!(components.scheme, Some("https".to_string()));
    assert_eq!(components.host, "example.com");
    assert_eq!(components.path, Some("/api".to_string()));
    assert_eq!(components.query, Some("limit=10".to_string()));
}

#[test]
fn test_parse_url_with_fragment() {
    let components = parse_url("https://example.com/page#section1").unwrap();
    assert_eq!(components.scheme, Some("https".to_string()));
    assert_eq!(components.host, "example.com");
    assert_eq!(components.path, Some("/page".to_string()));
    assert_eq!(components.fragment, Some("section1".to_string()));
}

#[test]
fn test_parse_url_complete() {
    let url = "https://user:pass@example.com:8080/api/v1?limit=10&offset=0#results";
    let components = parse_url(url).unwrap();

    assert_eq!(components.scheme, Some("https".to_string()));
    assert_eq!(components.username, Some("user".to_string()));
    assert_eq!(components.password, Some("pass".to_string()));
    assert_eq!(components.host, "example.com");
    assert_eq!(components.port, Some(8080));
    assert_eq!(components.path, Some("/api/v1".to_string()));
    assert_eq!(components.query, Some("limit=10&offset=0".to_string()));
    assert_eq!(components.fragment, Some("results".to_string()));
}

#[test]
fn test_parse_url_username_only() {
    let components = parse_url("https://user@example.com").unwrap();
    assert_eq!(components.username, Some("user".to_string()));
    assert_eq!(components.password, None);
}

#[test]
fn test_parse_url_invalid_port() {
    let result = parse_url("https://example.com:invalid");
    assert!(result.is_err());
}

#[test]
fn test_url_encode() {
    assert_eq!(url_encode("hello world"), "hello%20world");
    assert_eq!(url_encode("hello+world"), "hello%2Bworld");
    assert_eq!(url_encode("hello@example.com"), "hello%40example.com");
    assert_eq!(url_encode("safe-chars_123.~"), "safe-chars_123.~");
}

#[test]
fn test_url_encode_special_characters() {
    assert_eq!(url_encode("!@#$%^&*()"), "%21%40%23%24%25%5E%26%2A%28%29");
    assert_eq!(url_encode("{}[]|\\"), "%7B%7D%5B%5D%7C%5C");
}

#[test]
fn test_url_decode() {
    assert_eq!(url_decode("hello%20world").unwrap(), "hello world");
    assert_eq!(url_decode("hello%2Bworld").unwrap(), "hello+world");
    assert_eq!(
        url_decode("hello%40example.com").unwrap(),
        "hello@example.com"
    );
    assert_eq!(url_decode("safe-chars_123.~").unwrap(), "safe-chars_123.~");
}

#[test]
fn test_url_decode_plus_to_space() {
    assert_eq!(url_decode("hello+world").unwrap(), "hello world");
    assert_eq!(url_decode("form+data+test").unwrap(), "form data test");
}

#[test]
fn test_url_decode_invalid() {
    assert!(url_decode("hello%2").is_err()); // Incomplete encoding
    assert!(url_decode("hello%ZZ").is_err()); // Invalid hex
    assert!(url_decode("hello%").is_err()); // Incomplete encoding
}

#[test]
fn test_url_encode_decode_roundtrip() {
    let original = "Hello, World! @#$%^&*()";
    let encoded = url_encode(original);
    let decoded = url_decode(&encoded).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_url_components_default() {
    let components = UrlComponents::default();
    assert_eq!(components.host, "");
    assert_eq!(components.scheme, None);
    assert_eq!(components.port, None);
    assert_eq!(components.username, None);
    assert_eq!(components.password, None);
    assert_eq!(components.path, None);
    assert_eq!(components.query, None);
    assert_eq!(components.fragment, None);
}

#[test]
fn test_url_components_equality() {
    let comp1 = UrlComponents {
        scheme: Some("https".to_string()),
        host: "example.com".to_string(),
        port: Some(443),
        ..Default::default()
    };

    let comp2 = UrlComponents {
        scheme: Some("https".to_string()),
        host: "example.com".to_string(),
        port: Some(443),
        ..Default::default()
    };

    assert_eq!(comp1, comp2);
}

#[test]
fn test_builder_and_parser_roundtrip() {
    let original_url = UrlBuilder::new("example.com")
        .scheme("https")
        .username("user")
        .password("pass")
        .port(8080)
        .path("/api/v1")
        .query("key=value")
        .fragment("section")
        .build();

    let parsed = parse_url(&original_url).unwrap();

    let rebuilt = UrlBuilder::new(&parsed.host)
        .scheme(parsed.scheme.as_deref().unwrap_or(""))
        .username(parsed.username.as_deref().unwrap_or(""))
        .password(parsed.password.as_deref().unwrap_or(""))
        .port(parsed.port.unwrap_or(80))
        .path(parsed.path.as_deref().unwrap_or(""))
        .query(parsed.query.as_deref().unwrap_or(""))
        .fragment(parsed.fragment.as_deref().unwrap_or(""))
        .build();

    assert_eq!(original_url, rebuilt);
}

#[test]
fn test_edge_cases() {
    // Empty host
    let url = UrlBuilder::new("").build();
    assert_eq!(url, "");

    // Very long host
    let long_host = "a".repeat(1000);
    let url = UrlBuilder::new(&long_host).build();
    assert_eq!(url, long_host);

    // Special characters in host (should be handled by caller)
    let url = UrlBuilder::new("sub.example.com").build();
    assert_eq!(url, "sub.example.com");
}

#[test]
fn test_url_builder_only_host() {
    let builder = UrlBuilder::new("example.com");
    let url = builder.build();
    assert_eq!(url, "example.com");
}

#[test]
fn test_url_builder_with_all_options() {
    let builder = UrlBuilder::new("example.com")
        .scheme("https")
        .username("admin")
        .password("secret")
        .port(8443)
        .path("/api/v2/resource")
        .query("filter=active&sort=name")
        .fragment("top");

    let url = builder.build();
    assert!(url.contains("https://"));
    assert!(url.contains("admin:secret@"));
    assert!(url.contains("example.com:8443"));
    assert!(url.contains("/api/v2/resource"));
    assert!(url.contains("?filter=active&sort=name"));
    assert!(url.contains("#top"));
}

#[test]
fn test_parse_url_only_host() {
    let components = parse_url("example.com").unwrap();
    assert_eq!(components.host, "example.com");
    assert!(components.scheme.is_none());
    assert!(components.port.is_none());
    assert!(components.path.is_none());
}

#[test]
fn test_parse_url_with_scheme_and_host_only() {
    let components = parse_url("http://example.com").unwrap();
    assert_eq!(components.scheme, Some("http".to_string()));
    assert_eq!(components.host, "example.com");
    assert!(components.port.is_none());
}

#[test]
fn test_parse_url_with_port_no_path() {
    let components = parse_url("example.com:8080").unwrap();
    assert_eq!(components.host, "example.com");
    assert_eq!(components.port, Some(8080));
    assert!(components.path.is_none());
}

#[test]
fn test_parse_url_with_query_no_path() {
    let components = parse_url("example.com?key=value").unwrap();
    assert_eq!(components.host, "example.com");
    assert_eq!(components.query, Some("key=value".to_string()));
    assert!(components.path.is_none());
}

#[test]
fn test_parse_url_with_fragment_no_path() {
    let components = parse_url("example.com#section").unwrap();
    assert_eq!(components.host, "example.com");
    assert_eq!(components.fragment, Some("section".to_string()));
    assert!(components.path.is_none());
}

#[test]
fn test_parse_url_with_path_and_fragment_no_query() {
    let components = parse_url("example.com/page#section").unwrap();
    assert_eq!(components.host, "example.com");
    assert_eq!(components.path, Some("/page".to_string()));
    assert_eq!(components.fragment, Some("section".to_string()));
    assert!(components.query.is_none());
}

#[test]
fn test_parse_url_with_path_and_query_no_fragment() {
    let components = parse_url("example.com/api?key=value").unwrap();
    assert_eq!(components.host, "example.com");
    assert_eq!(components.path, Some("/api".to_string()));
    assert_eq!(components.query, Some("key=value".to_string()));
    assert!(components.fragment.is_none());
}

#[test]
fn test_url_encode_empty_string() {
    assert_eq!(url_encode(""), "");
}

#[test]
fn test_url_encode_only_safe_chars() {
    assert_eq!(url_encode("abc123-_.~"), "abc123-_.~");
}

#[test]
fn test_url_encode_all_special_chars() {
    let special = "!@#$%^&*()+={}[]|\\:;\"'<>,.?/`";
    let encoded = url_encode(special);
    assert!(!encoded.contains('!'));
    assert!(encoded.contains('%'));
}

#[test]
fn test_url_decode_empty_string() {
    assert_eq!(url_decode("").unwrap(), "");
}

#[test]
fn test_url_decode_only_safe_chars() {
    assert_eq!(url_decode("abc123-_.~").unwrap(), "abc123-_.~");
}

#[test]
fn test_url_decode_with_multiple_plus_signs() {
    assert_eq!(url_decode("hello+world+test").unwrap(), "hello world test");
}

#[test]
fn test_url_decode_mixed_encoding() {
    assert_eq!(
        url_decode("hello+world%20test").unwrap(),
        "hello world test"
    );
}

#[test]
fn test_url_decode_incomplete_at_end() {
    assert!(url_decode("test%2").is_err());
    assert!(url_decode("test%").is_err());
}

#[test]
fn test_url_decode_invalid_hex_chars() {
    assert!(url_decode("test%GG").is_err());
    assert!(url_decode("test%ZZ").is_err());
    assert!(url_decode("test%XY").is_err());
}

#[test]
fn test_url_components_all_fields() {
    let components = UrlComponents {
        scheme: Some("https".to_string()),
        username: Some("user".to_string()),
        password: Some("pass".to_string()),
        host: "example.com".to_string(),
        port: Some(443),
        path: Some("/api".to_string()),
        query: Some("key=value".to_string()),
        fragment: Some("section".to_string()),
    };

    assert_eq!(components.scheme, Some("https".to_string()));
    assert_eq!(components.username, Some("user".to_string()));
    assert_eq!(components.password, Some("pass".to_string()));
    assert_eq!(components.host, "example.com");
    assert_eq!(components.port, Some(443));
    assert_eq!(components.path, Some("/api".to_string()));
    assert_eq!(components.query, Some("key=value".to_string()));
    assert_eq!(components.fragment, Some("section".to_string()));
}

#[test]
fn test_url_components_clone() {
    let components = UrlComponents {
        scheme: Some("http".to_string()),
        host: "example.com".to_string(),
        ..Default::default()
    };

    let cloned = components.clone();
    assert_eq!(components, cloned);
}

#[test]
fn test_url_components_debug() {
    let components = UrlComponents::default();
    let debug_str = format!("{:?}", components);
    assert!(debug_str.contains("UrlComponents"));
}

#[test]
fn test_url_builder_partial_eq() {
    let builder1 = UrlBuilder::new("example.com").scheme("https").port(443);
    let builder2 = UrlBuilder::new("example.com").scheme("https").port(443);

    assert_eq!(builder1, builder2);
}

#[test]
fn test_parse_url_ipv4_address() {
    let components = parse_url("https://192.168.1.1:8080/path").unwrap();
    assert_eq!(components.scheme, Some("https".to_string()));
    assert_eq!(components.host, "192.168.1.1");
    assert_eq!(components.port, Some(8080));
    assert_eq!(components.path, Some("/path".to_string()));
}

#[test]
fn test_url_builder_query_and_fragment() {
    let url = UrlBuilder::new("example.com")
        .query("q=search")
        .fragment("results")
        .build();

    assert_eq!(url, "example.com?q=search#results");
}

#[test]
fn test_url_encode_unicode() {
    let encoded = url_encode("héllo");
    assert!(encoded.contains('%'));
    // Unicode characters should be encoded
}

#[test]
fn test_multiple_parse_and_build_combinations() {
    let test_urls = vec![
        "http://example.com",
        "https://user@example.com:8080",
        "example.com/path",
        "example.com?query=1",
        "example.com#fragment",
    ];

    for url in test_urls {
        let parsed = parse_url(url);
        assert!(parsed.is_ok(), "Failed to parse: {}", url);
    }
}
