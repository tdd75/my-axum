use std::collections::HashMap;
use tera::{Context, Tera};

pub fn render_email_template(
    template_path: &str,
    variables: HashMap<String, String>,
) -> anyhow::Result<String> {
    // Build the full path to the template file
    let full_path = format!("src/core/template/{}", template_path);
    let template_content = std::fs::read_to_string(&full_path)
        .map_err(|e| anyhow::anyhow!("Failed to read template file '{}': {}", full_path, e))?;

    let mut tera = Tera::default();
    tera.add_raw_template(template_path, &template_content)
        .map_err(|e| anyhow::anyhow!("Failed to add template '{}': {}", template_path, e))?;

    let mut context = Context::new();
    for (key, value) in variables {
        context.insert(key, &value);
    }

    tera.render(template_path, &context)
        .map_err(|e| anyhow::anyhow!("Failed to render template '{}': {}", template_path, e))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::render_email_template;

    fn welcome_variables() -> HashMap<String, String> {
        HashMap::from([
            ("app_name".to_string(), "Test App".to_string()),
            ("app_url".to_string(), "http://localhost:8000".to_string()),
            ("email".to_string(), "test@example.com".to_string()),
            ("first_name".to_string(), "John".to_string()),
            ("last_name".to_string(), "Doe".to_string()),
            ("phone".to_string(), "123456789".to_string()),
            ("year".to_string(), "2026".to_string()),
        ])
    }

    #[test]
    fn renders_existing_template() {
        let html = render_email_template("email/welcome.html", welcome_variables()).unwrap();
        assert!(html.contains("Test App"));
        assert!(html.contains("test@example.com"));
    }

    #[test]
    fn rejects_missing_template() {
        let error = render_email_template("email/missing.html", HashMap::new()).unwrap_err();
        assert!(error.to_string().contains("Failed to read template file"));
    }

    #[test]
    fn rejects_missing_variables() {
        let error = render_email_template("email/welcome.html", HashMap::new()).unwrap_err();
        assert!(error.to_string().contains("Failed to render template"));
    }
}
