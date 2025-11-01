#[derive(Clone, Debug)]
pub enum Language {
    English,
    Vietnamese,
}

impl Language {
    pub fn from_accept_language(header_value: &str) -> Self {
        // Parse Accept-Language header and find the language with highest quality value
        let mut languages: Vec<(&str, f32)> = header_value
            .split(',')
            .map(|lang| {
                let parts: Vec<&str> = lang.trim().split(';').collect();
                let lang_code = parts[0].trim();
                let quality = if parts.len() > 1 {
                    parts[1]
                        .trim()
                        .strip_prefix("q=")
                        .and_then(|q| q.parse::<f32>().ok())
                        .unwrap_or(1.0)
                } else {
                    1.0
                };
                (lang_code, quality)
            })
            .collect();

        // Sort by quality value in descending order
        languages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return the first matching language
        for (lang_code, _) in languages {
            if lang_code.starts_with("vi") {
                return Language::Vietnamese;
            } else if lang_code.starts_with("en") {
                return Language::English;
            }
        }

        // Default to English if no match found
        Language::English
    }

    pub fn to_locale(&self) -> &str {
        match self {
            Language::English => "en",
            Language::Vietnamese => "vi",
        }
    }
}
