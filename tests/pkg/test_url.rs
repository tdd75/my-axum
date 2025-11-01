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
