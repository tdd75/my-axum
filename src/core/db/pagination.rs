pub fn calculate_offset(page: Option<u64>, page_size: u64) -> u64 {
    match page {
        Some(p) if p > 0 => (p - 1) * page_size,
        _ => 0,
    }
}
