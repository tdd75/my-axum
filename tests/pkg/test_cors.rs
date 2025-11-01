use my_axum::pkg::cors::*;

#[test]
fn test_exact_match() {
    assert!(matches_origin_pattern(
        "https://example.com",
        "https://example.com"
    ));
    assert!(!matches_origin_pattern(
        "https://example.com",
        "https://other.com"
    ));
    assert!(!matches_origin_pattern(
        "https://app.example.com",
        "https://example.com"
    ));
}

#[test]
fn test_wildcard_all() {
    assert!(matches_origin_pattern("https://example.com", "*"));
    assert!(matches_origin_pattern("http://localhost:3000", "*"));
    assert!(matches_origin_pattern("https://any.domain.com", "*"));
}

#[test]
fn test_subdomain_wildcard() {
    let pattern = "https://*.example.com";

    // Should match subdomains
    assert!(matches_origin_pattern("https://app.example.com", pattern));
    assert!(matches_origin_pattern("https://api.example.com", pattern));
    assert!(matches_origin_pattern(
        "https://sub.domain.example.com",
        pattern
    ));
    assert!(matches_origin_pattern("https://a.b.c.example.com", pattern));

    // Should not match base domain
    assert!(!matches_origin_pattern("https://example.com", pattern));

    // Should not match different domains
    assert!(!matches_origin_pattern("https://example.org", pattern));
    assert!(!matches_origin_pattern("https://notexample.com", pattern));

    // Should not match different protocols
    assert!(!matches_origin_pattern("http://app.example.com", pattern));
}

#[test]
fn test_protocol_wildcard() {
    let pattern = "*://example.com";

    assert!(matches_origin_pattern("https://example.com", pattern));
    assert!(matches_origin_pattern("http://example.com", pattern));
    assert!(!matches_origin_pattern("https://app.example.com", pattern));
}

#[test]
fn test_protocol_wildcard_with_subdomain() {
    let pattern = "*://*.example.com";

    assert!(matches_origin_pattern("https://app.example.com", pattern));
    assert!(matches_origin_pattern("http://app.example.com", pattern));
    assert!(matches_origin_pattern("https://api.example.com", pattern));
    assert!(!matches_origin_pattern("https://example.com", pattern));
}

#[test]
fn test_wildcard_host_only() {
    let pattern = "https://*";

    assert!(matches_origin_pattern("https://example.com", pattern));
    assert!(matches_origin_pattern("https://any.domain.com", pattern));
    assert!(!matches_origin_pattern("http://example.com", pattern));
}

#[test]
fn test_port_in_origin() {
    assert!(matches_origin_pattern(
        "http://localhost:3000",
        "http://localhost:3000"
    ));
    assert!(!matches_origin_pattern(
        "http://localhost:3000",
        "http://localhost:8000"
    ));
}

#[test]
fn test_invalid_patterns() {
    // Pattern without protocol separator
    assert!(!matches_origin_pattern(
        "https://example.com",
        "example.com"
    ));

    // Origin without protocol separator
    assert!(!matches_origin_pattern(
        "example.com",
        "https://example.com"
    ));

    // Wildcard in unsupported position
    assert!(!matches_origin_pattern(
        "https://example.com",
        "https://example.*"
    ));
}

#[test]
fn test_case_sensitivity() {
    // URLs should be case-sensitive
    assert!(!matches_origin_pattern(
        "https://Example.com",
        "https://example.com"
    ));
    assert!(matches_origin_pattern(
        "https://Example.com",
        "https://Example.com"
    ));
}

#[test]
fn test_trailing_slash() {
    // Should not match with trailing slash difference
    assert!(!matches_origin_pattern(
        "https://example.com/",
        "https://example.com"
    ));
    assert!(matches_origin_pattern(
        "https://example.com/",
        "https://example.com/"
    ));
}

#[test]
fn test_multiple_subdomains() {
    let pattern = "https://*.example.com";

    assert!(matches_origin_pattern("https://a.example.com", pattern));
    assert!(matches_origin_pattern("https://a.b.example.com", pattern));
    assert!(matches_origin_pattern("https://a.b.c.example.com", pattern));
    assert!(matches_origin_pattern(
        "https://very.long.subdomain.chain.example.com",
        pattern
    ));
}

#[test]
fn test_wildcard_with_port() {
    let pattern = "https://*.example.com:8443";

    assert!(matches_origin_pattern(
        "https://app.example.com:8443",
        pattern
    ));
    assert!(!matches_origin_pattern("https://app.example.com", pattern));
    assert!(!matches_origin_pattern(
        "https://app.example.com:443",
        pattern
    ));
}

#[test]
fn test_empty_strings() {
    assert!(!matches_origin_pattern("", "https://example.com"));
    assert!(!matches_origin_pattern("https://example.com", ""));
    assert!(!matches_origin_pattern("", ""));
}

#[test]
fn test_real_world_patterns() {
    // Common real-world scenarios
    let pattern = "https://*.myapp.com";

    assert!(matches_origin_pattern("https://staging.myapp.com", pattern));
    assert!(matches_origin_pattern(
        "https://production.myapp.com",
        pattern
    ));
    assert!(matches_origin_pattern("https://dev.myapp.com", pattern));
    assert!(matches_origin_pattern(
        "https://api.staging.myapp.com",
        pattern
    ));

    assert!(!matches_origin_pattern("https://myapp.com", pattern));
    assert!(!matches_origin_pattern("https://fakemyapp.com", pattern));
    assert!(!matches_origin_pattern("http://staging.myapp.com", pattern));
}
