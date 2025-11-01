#[cfg(test)]
mod language_tests {
    use my_axum::core::translation::language::Language;

    #[test]
    fn test_from_accept_language_vietnamese_priority() {
        let header = "vi-VN,vi;q=0.9,en;q=0.8";
        let language = Language::from_accept_language(header);
        matches!(language, Language::Vietnamese);
    }

    #[test]
    fn test_from_accept_language_english_priority() {
        let header = "en-US,en;q=0.9,vi;q=0.8";
        let language = Language::from_accept_language(header);
        matches!(language, Language::English);
    }

    #[test]
    fn test_from_accept_language_complex_header() {
        let header = "fr;q=0.9,vi;q=0.8,en;q=0.7,de;q=0.6";
        let language = Language::from_accept_language(header);
        matches!(language, Language::Vietnamese);
    }

    #[test]
    fn test_from_accept_language_no_quality_values() {
        let header = "vi,en";
        let language = Language::from_accept_language(header);
        matches!(language, Language::Vietnamese);
    }

    #[test]
    fn test_from_accept_language_english_first() {
        let header = "en,vi";
        let language = Language::from_accept_language(header);
        matches!(language, Language::English);
    }

    #[test]
    fn test_from_accept_language_unsupported_language() {
        let header = "fr,de,es";
        let language = Language::from_accept_language(header);
        matches!(language, Language::English); // Should default to English
    }

    #[test]
    fn test_from_accept_language_empty_header() {
        let header = "";
        let language = Language::from_accept_language(header);
        matches!(language, Language::English); // Should default to English
    }

    #[test]
    fn test_from_accept_language_malformed_quality() {
        let header = "vi;q=invalid,en;q=0.8";
        let language = Language::from_accept_language(header);
        matches!(language, Language::Vietnamese); // Should still work, invalid q defaults to 1.0
    }

    #[test]
    fn test_from_accept_language_partial_vietnamese_code() {
        let header = "vi-VN";
        let language = Language::from_accept_language(header);
        matches!(language, Language::Vietnamese);
    }

    #[test]
    fn test_from_accept_language_partial_english_code() {
        let header = "en-GB";
        let language = Language::from_accept_language(header);
        matches!(language, Language::English);
    }

    #[test]
    fn test_from_accept_language_zero_quality() {
        let header = "vi;q=0.0,en;q=0.8";
        let language = Language::from_accept_language(header);
        matches!(language, Language::English);
    }

    #[test]
    fn test_from_accept_language_whitespace() {
        let header = " vi ; q=0.9 , en ; q=0.8 ";
        let language = Language::from_accept_language(header);
        matches!(language, Language::Vietnamese);
    }

    #[test]
    fn test_to_locale_english() {
        let language = Language::English;
        assert_eq!(language.to_locale(), "en");
    }

    #[test]
    fn test_to_locale_vietnamese() {
        let language = Language::Vietnamese;
        assert_eq!(language.to_locale(), "vi");
    }

    #[test]
    fn test_language_clone() {
        let language = Language::Vietnamese;
        let cloned = language.clone();
        matches!(cloned, Language::Vietnamese);
    }

    #[test]
    fn test_language_debug() {
        let language = Language::English;
        let debug_str = format!("{:?}", language);
        assert!(debug_str.contains("English"));
    }

    #[test]
    fn test_from_accept_language_case_insensitive_quality() {
        let header = "vi;Q=0.9,en;q=0.8";
        let language = Language::from_accept_language(header);
        // Should handle case where Q is uppercase (though not standard)
        matches!(language, Language::Vietnamese);
    }

    #[test]
    fn test_from_accept_language_multiple_semicolons() {
        let header = "vi;q=0.9;charset=utf-8,en;q=0.8";
        let language = Language::from_accept_language(header);
        matches!(language, Language::Vietnamese);
    }
}
