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
