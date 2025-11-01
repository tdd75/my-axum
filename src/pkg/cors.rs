/// Check if an origin matches a pattern with wildcard support
/// Supports patterns like:
/// - `https://example.com` (exact match)
/// - `https://*.example.com` (subdomain wildcard)
/// - `*` (allow all - use with caution)
pub fn matches_origin_pattern(origin: &str, pattern: &str) -> bool {
    // Empty strings should not match
    if origin.is_empty() || pattern.is_empty() {
        return false;
    }

    // Allow all origins
    if pattern == "*" {
        return true;
    }

    // Exact match
    if pattern == origin {
        return true;
    }

    // Wildcard pattern matching
    if pattern.contains('*') {
        // Convert wildcard pattern to regex-like matching
        // https://*.example.com -> should match https://sub.example.com, https://a.b.example.com
        let pattern_parts: Vec<&str> = pattern.split("://").collect();
        let origin_parts: Vec<&str> = origin.split("://").collect();

        if pattern_parts.len() != 2 || origin_parts.len() != 2 {
            return false;
        }

        // Check protocol matches
        let (pattern_protocol, pattern_host) = (pattern_parts[0], pattern_parts[1]);
        let (origin_protocol, origin_host) = (origin_parts[0], origin_parts[1]);

        // Protocol wildcard or exact match
        if pattern_protocol == "*" || pattern_protocol == origin_protocol {
            // Handle wildcard in host
            if let Some(pattern_domain) = pattern_host.strip_prefix("*.") {
                // Origin must end with .pattern_domain and have at least one subdomain
                // Check if origin_host ends with the pattern domain
                if !origin_host.ends_with(pattern_domain) {
                    return false;
                }
                // Make sure it's actually a subdomain, not just matching the end
                // e.g., "fakemyapp.com" should not match "*.myapp.com"
                let prefix_len = origin_host.len() - pattern_domain.len();
                if prefix_len == 0 {
                    // origin_host == pattern_domain, no subdomain
                    return false;
                }
                // Check if there's a dot before the domain part
                origin_host.chars().nth(prefix_len - 1) == Some('.')
            } else if pattern_host == "*" {
                true
            } else {
                // No wildcard in host, so it must be exact match
                pattern_host == origin_host
            }
        } else {
            false
        }
    } else {
        false
    }
}
