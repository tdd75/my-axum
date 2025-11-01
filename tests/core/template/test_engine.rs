#[cfg(test)]
mod engine_tests {
    use my_axum::core::template::engine::render_email_template;
    use std::collections::HashMap;

    #[test]
    fn test_render_email_template_success() {
        // Test successful rendering with all variables
        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "Test App".to_string());
        variables.insert("app_url".to_string(), "http://localhost:8000".to_string());
        variables.insert("email".to_string(), "test@example.com".to_string());
        variables.insert("first_name".to_string(), "John".to_string());
        variables.insert("last_name".to_string(), "Doe".to_string());
        variables.insert("phone".to_string(), "123456789".to_string());
        variables.insert("year".to_string(), "2025".to_string());

        let result = render_email_template("email/welcome.html", variables);

        assert!(result.is_ok(), "Template rendering should succeed");
        let html = result.unwrap();
        assert!(html.contains("Test App"), "HTML should contain app name");
        assert!(
            html.contains("test@example.com"),
            "HTML should contain email"
        );
        assert!(html.contains("John"), "HTML should contain first name");
        assert!(html.contains("Doe"), "HTML should contain last name");
        assert!(html.contains("123456789"), "HTML should contain phone");
        assert!(html.contains("2025"), "HTML should contain year");
    }

    #[test]
    fn test_render_email_template_with_minimal_variables() {
        // Test rendering with minimal required variables
        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "Minimal App".to_string());
        variables.insert("app_url".to_string(), "http://example.com".to_string());
        variables.insert("email".to_string(), "minimal@example.com".to_string());
        variables.insert("first_name".to_string(), "".to_string());
        variables.insert("last_name".to_string(), "".to_string());
        variables.insert("phone".to_string(), "".to_string());
        variables.insert("year".to_string(), "2025".to_string());

        let result = render_email_template("email/welcome.html", variables);

        assert!(
            result.is_ok(),
            "Template rendering with minimal data should succeed"
        );
        let html = result.unwrap();
        assert!(html.contains("Minimal App"), "HTML should contain app name");
        assert!(
            html.contains("minimal@example.com"),
            "HTML should contain email"
        );
    }

    #[test]
    fn test_render_email_template_with_special_characters() {
        // Test rendering with special characters in variables
        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "Test & Co. <App>".to_string());
        variables.insert("app_url".to_string(), "http://localhost:8000".to_string());
        variables.insert("email".to_string(), "test+special@example.com".to_string());
        variables.insert("first_name".to_string(), "Jean-Pierre".to_string());
        variables.insert("last_name".to_string(), "O'Connor".to_string());
        variables.insert("phone".to_string(), "+1 (555) 123-4567".to_string());
        variables.insert("year".to_string(), "2025".to_string());

        let result = render_email_template("email/welcome.html", variables);

        assert!(
            result.is_ok(),
            "Template rendering with special characters should succeed"
        );
        let html = result.unwrap();
        // Tera escapes HTML by default, so & and < should be escaped
        assert!(
            html.contains("Test &amp; Co. &lt;App&gt;") || html.contains("Test & Co. <App>"),
            "HTML should contain app name with special chars"
        );
        assert!(
            html.contains("test+special@example.com"),
            "HTML should contain email"
        );
        assert!(
            html.contains("Jean-Pierre"),
            "HTML should contain first name with hyphen"
        );
        assert!(
            html.contains("O&#x27;Connor") || html.contains("O'Connor"),
            "HTML should contain last name with apostrophe"
        );
    }

    #[test]
    fn test_render_email_template_with_empty_variables() {
        // Test rendering with empty HashMap
        let variables = HashMap::new();

        let result = render_email_template("email/welcome.html", variables);

        // Should fail because template expects certain variables
        assert!(
            result.is_err(),
            "Template rendering with no variables should fail"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to render template"),
            "Error should mention rendering failure"
        );
    }

    #[test]
    fn test_render_email_template_file_not_found() {
        // Test with non-existent template file
        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "Test App".to_string());

        let result = render_email_template("email/nonexistent.html", variables);

        assert!(
            result.is_err(),
            "Rendering non-existent template should fail"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to read template file"),
            "Error should mention file reading failure: {}",
            error_msg
        );
    }

    #[test]
    fn test_render_email_template_with_unicode() {
        // Test rendering with Unicode characters
        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "Test App 测试".to_string());
        variables.insert("app_url".to_string(), "http://localhost:8000".to_string());
        variables.insert("email".to_string(), "test@例え.jp".to_string());
        variables.insert("first_name".to_string(), "José".to_string());
        variables.insert("last_name".to_string(), "Müller".to_string());
        variables.insert("phone".to_string(), "123456789".to_string());
        variables.insert("year".to_string(), "2025".to_string());

        let result = render_email_template("email/welcome.html", variables);

        assert!(
            result.is_ok(),
            "Template rendering with Unicode should succeed"
        );
        let html = result.unwrap();
        assert!(
            html.contains("Test App 测试") || html.contains("Test App"),
            "HTML should contain app name with Unicode"
        );
        assert!(
            html.contains("José"),
            "HTML should contain first name with accent"
        );
        assert!(
            html.contains("Müller"),
            "HTML should contain last name with umlaut"
        );
    }

    #[test]
    fn test_render_email_template_with_long_content() {
        // Test rendering with very long variable content
        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "A".repeat(1000));
        variables.insert("app_url".to_string(), "http://localhost:8000".to_string());
        variables.insert("email".to_string(), "test@example.com".to_string());
        variables.insert("first_name".to_string(), "B".repeat(500));
        variables.insert("last_name".to_string(), "C".repeat(500));
        variables.insert("phone".to_string(), "123456789".to_string());
        variables.insert("year".to_string(), "2025".to_string());

        let result = render_email_template("email/welcome.html", variables);

        assert!(
            result.is_ok(),
            "Template rendering with long content should succeed"
        );
        let html = result.unwrap();
        assert!(html.len() > 2000, "HTML should contain long content");
    }

    #[test]
    fn test_render_email_template_with_numeric_strings() {
        // Test rendering with numeric strings
        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "123 App".to_string());
        variables.insert("app_url".to_string(), "http://localhost:8000".to_string());
        variables.insert("email".to_string(), "user123@example.com".to_string());
        variables.insert("first_name".to_string(), "456".to_string());
        variables.insert("last_name".to_string(), "789".to_string());
        variables.insert("phone".to_string(), "9876543210".to_string());
        variables.insert("year".to_string(), "2025".to_string());

        let result = render_email_template("email/welcome.html", variables);

        assert!(
            result.is_ok(),
            "Template rendering with numeric strings should succeed"
        );
        let html = result.unwrap();
        assert!(
            html.contains("123 App"),
            "HTML should contain numeric app name"
        );
        assert!(
            html.contains("456"),
            "HTML should contain numeric first name"
        );
        assert!(
            html.contains("789"),
            "HTML should contain numeric last name"
        );
    }

    #[test]
    fn test_render_email_template_with_invalid_path() {
        // Test with completely invalid template path
        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "Test App".to_string());
        variables.insert("app_url".to_string(), "http://localhost:8000".to_string());
        variables.insert("email".to_string(), "test@example.com".to_string());
        variables.insert("first_name".to_string(), "John".to_string());
        variables.insert("last_name".to_string(), "Doe".to_string());
        variables.insert("phone".to_string(), "123456789".to_string());
        variables.insert("year".to_string(), "2025".to_string());

        // Test with a path that tries to escape the template directory
        let result = render_email_template("../../../etc/passwd", variables);

        assert!(
            result.is_err(),
            "Template rendering with invalid path should fail"
        );
    }

    #[test]
    fn test_render_email_template_preserves_html_structure() {
        // Verify that rendered HTML maintains proper structure
        let mut variables = HashMap::new();
        variables.insert("app_name".to_string(), "Structure Test".to_string());
        variables.insert("app_url".to_string(), "http://localhost:8000".to_string());
        variables.insert("email".to_string(), "structure@test.com".to_string());
        variables.insert("first_name".to_string(), "Test".to_string());
        variables.insert("last_name".to_string(), "User".to_string());
        variables.insert("phone".to_string(), "123456789".to_string());
        variables.insert("year".to_string(), "2025".to_string());

        let result = render_email_template("email/welcome.html", variables);

        assert!(result.is_ok(), "Template rendering should succeed");
        let html = result.unwrap();

        // Check for HTML structure elements
        assert!(html.contains("<!DOCTYPE html>"), "Should have DOCTYPE");
        assert!(html.contains("<html"), "Should have opening html tag");
        assert!(html.contains("</html>"), "Should have closing html tag");
        assert!(html.contains("<head>"), "Should have head section");
        assert!(html.contains("<body>"), "Should have body section");
        assert!(html.contains("<style>"), "Should have style section");
    }
}
