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

#[cfg(test)]
mod tests {
    use super::Language;

    #[test]
    fn picks_language_by_quality() {
        assert!(matches!(
            Language::from_accept_language("vi-VN,vi;q=0.9,en;q=0.8"),
            Language::Vietnamese
        ));
        assert!(matches!(
            Language::from_accept_language("en-US,en;q=0.9,vi;q=0.8"),
            Language::English
        ));
    }

    #[test]
    fn defaults_to_english() {
        assert!(matches!(
            Language::from_accept_language("fr,de"),
            Language::English
        ));
        assert!(matches!(
            Language::from_accept_language(""),
            Language::English
        ));
    }

    #[test]
    fn converts_to_locale() {
        assert_eq!(Language::English.to_locale(), "en");
        assert_eq!(Language::Vietnamese.to_locale(), "vi");
    }
}
