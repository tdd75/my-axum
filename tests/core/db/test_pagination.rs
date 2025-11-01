#[cfg(test)]
mod pagination_tests {
    use my_axum::core::db::pagination::calculate_offset;

    #[test]
    fn test_calculate_offset_first_page() {
        let offset = calculate_offset(Some(1), 10);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_calculate_offset_second_page() {
        let offset = calculate_offset(Some(2), 10);
        assert_eq!(offset, 10);
    }

    #[test]
    fn test_calculate_offset_third_page() {
        let offset = calculate_offset(Some(3), 10);
        assert_eq!(offset, 20);
    }

    #[test]
    fn test_calculate_offset_page_size_20() {
        let offset = calculate_offset(Some(2), 20);
        assert_eq!(offset, 20);
    }

    #[test]
    fn test_calculate_offset_page_size_50() {
        let offset = calculate_offset(Some(3), 50);
        assert_eq!(offset, 100);
    }

    #[test]
    fn test_calculate_offset_large_page_number() {
        let offset = calculate_offset(Some(100), 10);
        assert_eq!(offset, 990);
    }

    #[test]
    fn test_calculate_offset_none_page() {
        let offset = calculate_offset(None, 10);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_calculate_offset_zero_page() {
        let offset = calculate_offset(Some(0), 10);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_calculate_offset_page_size_1() {
        let offset = calculate_offset(Some(5), 1);
        assert_eq!(offset, 4);
    }

    #[test]
    fn test_calculate_offset_page_size_100() {
        let offset = calculate_offset(Some(2), 100);
        assert_eq!(offset, 100);
    }

    #[test]
    fn test_calculate_offset_various_combinations() {
        // Test multiple page and page_size combinations
        assert_eq!(calculate_offset(Some(1), 5), 0);
        assert_eq!(calculate_offset(Some(2), 5), 5);
        assert_eq!(calculate_offset(Some(3), 5), 10);
        assert_eq!(calculate_offset(Some(4), 5), 15);
        assert_eq!(calculate_offset(Some(5), 5), 20);
    }

    #[test]
    fn test_calculate_offset_formula_verification() {
        // Verify the formula: offset = (page - 1) * page_size
        let page = 7;
        let page_size = 15;
        let expected = (page - 1) * page_size;
        let actual = calculate_offset(Some(page), page_size);

        assert_eq!(actual, expected);
        assert_eq!(actual, 90);
    }

    #[test]
    fn test_calculate_offset_edge_case_page_1_size_1() {
        let offset = calculate_offset(Some(1), 1);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_calculate_offset_edge_case_large_page_size() {
        let offset = calculate_offset(Some(2), 1000);
        assert_eq!(offset, 1000);
    }

    #[test]
    fn test_calculate_offset_consistency_across_calls() {
        let page = Some(5);
        let page_size = 20;

        let offset1 = calculate_offset(page, page_size);
        let offset2 = calculate_offset(page, page_size);

        assert_eq!(offset1, offset2);
    }

    #[test]
    fn test_calculate_offset_sequential_pages() {
        let page_size = 25;

        let offset1 = calculate_offset(Some(1), page_size);
        let offset2 = calculate_offset(Some(2), page_size);
        let offset3 = calculate_offset(Some(3), page_size);

        // Each offset should increase by page_size
        assert_eq!(offset2 - offset1, page_size);
        assert_eq!(offset3 - offset2, page_size);
    }

    #[test]
    fn test_calculate_offset_different_page_sizes_same_page() {
        let page = Some(3);

        let offset_10 = calculate_offset(page, 10);
        let offset_20 = calculate_offset(page, 20);
        let offset_30 = calculate_offset(page, 30);

        assert_eq!(offset_10, 20);
        assert_eq!(offset_20, 40);
        assert_eq!(offset_30, 60);
    }

    #[test]
    fn test_calculate_offset_typical_pagination_scenario() {
        // Simulating typical pagination: 10 items per page
        let page_size = 10;

        // First page (items 1-10)
        assert_eq!(calculate_offset(Some(1), page_size), 0);

        // Second page (items 11-20)
        assert_eq!(calculate_offset(Some(2), page_size), 10);

        // Third page (items 21-30)
        assert_eq!(calculate_offset(Some(3), page_size), 20);
    }

    #[test]
    fn test_calculate_offset_typical_pagination_20_per_page() {
        // Simulating pagination: 20 items per page
        let page_size = 20;

        assert_eq!(calculate_offset(Some(1), page_size), 0); // Items 1-20
        assert_eq!(calculate_offset(Some(2), page_size), 20); // Items 21-40
        assert_eq!(calculate_offset(Some(3), page_size), 40); // Items 41-60
        assert_eq!(calculate_offset(Some(4), page_size), 60); // Items 61-80
    }

    #[test]
    fn test_calculate_offset_typical_pagination_50_per_page() {
        // Simulating pagination: 50 items per page
        let page_size = 50;

        assert_eq!(calculate_offset(Some(1), page_size), 0); // Items 1-50
        assert_eq!(calculate_offset(Some(2), page_size), 50); // Items 51-100
        assert_eq!(calculate_offset(Some(3), page_size), 100); // Items 101-150
    }

    #[test]
    fn test_calculate_offset_boundary_conditions() {
        // Test boundary conditions
        assert_eq!(calculate_offset(Some(1), u64::MAX), 0);
        assert_eq!(calculate_offset(None, u64::MAX), 0);
        assert_eq!(calculate_offset(Some(0), u64::MAX), 0);
    }

    #[test]
    fn test_calculate_offset_with_typical_rest_api_pagination() {
        // Common REST API pagination patterns

        // Mobile app: 20 items per page
        assert_eq!(calculate_offset(Some(1), 20), 0);
        assert_eq!(calculate_offset(Some(2), 20), 20);

        // Web app: 50 items per page
        assert_eq!(calculate_offset(Some(1), 50), 0);
        assert_eq!(calculate_offset(Some(2), 50), 50);

        // Admin panel: 100 items per page
        assert_eq!(calculate_offset(Some(1), 100), 0);
        assert_eq!(calculate_offset(Some(2), 100), 100);
    }

    #[test]
    fn test_calculate_offset_matches_expected_sql_offset() {
        // Verify that our offset matches what you'd use in SQL OFFSET clause
        let page = 3;
        let page_size = 15;
        let offset = calculate_offset(Some(page), page_size);

        // In SQL: SELECT * FROM table LIMIT 15 OFFSET 30
        assert_eq!(offset, 30);
    }

    #[test]
    fn test_calculate_offset_return_type() {
        // Ensure the function returns u64 type
        let result: u64 = calculate_offset(Some(1), 10);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_calculate_offset_no_overflow_reasonable_values() {
        // Test with reasonable values that shouldn't overflow
        let offset = calculate_offset(Some(1000), 100);
        assert_eq!(offset, 99900);
    }

    #[test]
    fn test_calculate_offset_consistency_with_skip_take() {
        // Verify it works well with skip/take pattern
        let page = 4;
        let page_size = 25;
        let offset = calculate_offset(Some(page), page_size);

        // This offset would be used like: query.skip(offset).take(page_size)
        assert_eq!(offset, 75); // Skip first 75 items, take next 25
    }

    #[test]
    fn test_calculate_offset_zero_based_vs_one_based() {
        // Our function uses 1-based page numbers (page 1 is first page)
        assert_eq!(calculate_offset(Some(1), 10), 0); // First page starts at 0
        assert_eq!(calculate_offset(Some(2), 10), 10); // Second page starts at 10

        // Not zero-based (page 0 would be treated as invalid/None)
        assert_eq!(calculate_offset(Some(0), 10), 0); // Invalid page treated as first page
    }

    #[test]
    fn test_calculate_offset_practical_examples() {
        // E-commerce product listing: 24 items per page
        assert_eq!(calculate_offset(Some(1), 24), 0);
        assert_eq!(calculate_offset(Some(2), 24), 24);
        assert_eq!(calculate_offset(Some(3), 24), 48);

        // Blog posts: 10 posts per page
        assert_eq!(calculate_offset(Some(1), 10), 0);
        assert_eq!(calculate_offset(Some(5), 10), 40);

        // Search results: 15 results per page
        assert_eq!(calculate_offset(Some(1), 15), 0);
        assert_eq!(calculate_offset(Some(3), 15), 30);
    }
}
