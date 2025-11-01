pub fn calculate_offset(page: Option<u64>, page_size: u64) -> u64 {
    match page {
        Some(p) if p > 0 => (p - 1) * page_size,
        _ => 0,
    }
}

pub fn clamp_page_size(page_size: Option<u64>, page_size_limit: Option<u64>) -> Option<u64> {
    match (page_size, page_size_limit) {
        (Some(page_size), Some(limit)) => Some(page_size.min(limit)),
        (page_size, _) => page_size,
    }
}

#[cfg(test)]
mod tests {
    use super::{calculate_offset, clamp_page_size};

    #[test]
    fn calculates_offsets() {
        assert_eq!(calculate_offset(Some(1), 10), 0);
        assert_eq!(calculate_offset(Some(2), 10), 10);
        assert_eq!(calculate_offset(Some(3), 25), 50);
    }

    #[test]
    fn handles_missing_or_zero_page() {
        assert_eq!(calculate_offset(None, 10), 0);
        assert_eq!(calculate_offset(Some(0), 10), 0);
    }

    #[test]
    fn clamps_page_size_only_when_limit_is_configured() {
        assert_eq!(clamp_page_size(Some(50), Some(20)), Some(20));
        assert_eq!(clamp_page_size(Some(10), Some(20)), Some(10));
        assert_eq!(clamp_page_size(Some(50), None), Some(50));
        assert_eq!(clamp_page_size(None, Some(20)), None);
    }
}
