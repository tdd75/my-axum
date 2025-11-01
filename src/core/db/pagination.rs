pub fn calculate_offset(page: Option<u64>, page_size: u64) -> u64 {
    match page {
        Some(p) if p > 0 => (p - 1) * page_size,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::calculate_offset;

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
}
