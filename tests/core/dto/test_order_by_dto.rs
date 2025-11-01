#[cfg(test)]
mod order_by_dto_tests {
    use my_axum::core::db::ordering::{OrderBy, OrderByField, SortOrder};

    // Mock OrderByField implementation for testing
    #[derive(Debug, Clone, PartialEq)]
    enum TestField {
        Id,
        Name,
        CreatedAt,
        UpdatedAt,
    }

    impl OrderByField for TestField {
        fn from_str(s: &str) -> Option<Self> {
            match s {
                "id" => Some(TestField::Id),
                "name" => Some(TestField::Name),
                "created_at" => Some(TestField::CreatedAt),
                "updated_at" => Some(TestField::UpdatedAt),
                _ => None,
            }
        }

        fn to_string(&self) -> String {
            match self {
                TestField::Id => "id".to_string(),
                TestField::Name => "name".to_string(),
                TestField::CreatedAt => "created_at".to_string(),
                TestField::UpdatedAt => "updated_at".to_string(),
            }
        }
    }

    #[test]
    fn test_sort_order_default() {
        let sort_order = SortOrder::default();
        assert!(matches!(sort_order, SortOrder::Asc));
    }

    #[test]
    fn test_sort_order_clone() {
        let sort_order = SortOrder::Desc;
        let cloned = sort_order.clone();
        assert!(matches!(cloned, SortOrder::Desc));
        assert!(sort_order == cloned);
    }

    #[test]
    fn test_order_by_new() {
        let order_by = OrderBy::new(TestField::Id, SortOrder::Asc);
        assert_eq!(order_by.field, TestField::Id);
        assert!(matches!(order_by.order, SortOrder::Asc));
    }

    #[test]
    fn test_order_by_new_desc() {
        let order_by = OrderBy::new(TestField::Name, SortOrder::Desc);
        assert_eq!(order_by.field, TestField::Name);
        assert!(matches!(order_by.order, SortOrder::Desc));
    }

    #[test]
    fn test_parse_order_by_string_single_ascending() {
        let order_by_str = "+id";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].field, TestField::Id);
        assert!(matches!(result[0].order, SortOrder::Asc));
    }

    #[test]
    fn test_parse_order_by_string_single_descending() {
        let order_by_str = "-name";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].field, TestField::Name);
        assert!(matches!(result[0].order, SortOrder::Desc));
    }

    #[test]
    fn test_parse_order_by_string_single_default_ascending() {
        let order_by_str = "created_at";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].field, TestField::CreatedAt);
        assert!(matches!(result[0].order, SortOrder::Asc));
    }

    #[test]
    fn test_parse_order_by_string_multiple_fields() {
        let order_by_str = "+name,-created_at,id";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 3);

        assert_eq!(result[0].field, TestField::Name);
        assert!(matches!(result[0].order, SortOrder::Asc));

        assert_eq!(result[1].field, TestField::CreatedAt);
        assert!(matches!(result[1].order, SortOrder::Desc));

        assert_eq!(result[2].field, TestField::Id);
        assert!(matches!(result[2].order, SortOrder::Asc));
    }

    #[test]
    fn test_parse_order_by_string_with_spaces() {
        let order_by_str = " +name , -created_at , id ";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].field, TestField::Name);
        assert!(matches!(result[0].order, SortOrder::Asc));
    }

    #[test]
    fn test_parse_order_by_string_empty() {
        let order_by_str = "";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_order_by_string_invalid_field() {
        let order_by_str = "+invalid_field,name";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].field, TestField::Name);
        assert!(matches!(result[0].order, SortOrder::Asc));
    }

    #[test]
    fn test_parse_order_by_string_empty_parts() {
        let order_by_str = "+name,,,-created_at,";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].field, TestField::Name);
        assert!(matches!(result[0].order, SortOrder::Asc));
        assert_eq!(result[1].field, TestField::CreatedAt);
        assert!(matches!(result[1].order, SortOrder::Desc));
    }

    #[test]
    fn test_parse_order_by_string_only_commas() {
        let order_by_str = ",,,";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_order_by_clone() {
        let original = OrderBy::new(TestField::Id, SortOrder::Desc);
        let cloned = original.clone();

        assert_eq!(original.field, cloned.field);
        assert!(original.order == cloned.order);
    }

    #[test]
    fn test_test_field_from_str_valid() {
        assert_eq!(TestField::from_str("id"), Some(TestField::Id));
        assert_eq!(TestField::from_str("name"), Some(TestField::Name));
        assert_eq!(
            TestField::from_str("created_at"),
            Some(TestField::CreatedAt)
        );
        assert_eq!(
            TestField::from_str("updated_at"),
            Some(TestField::UpdatedAt)
        );
    }

    #[test]
    fn test_test_field_from_str_invalid() {
        assert_eq!(TestField::from_str("invalid"), None);
        assert_eq!(TestField::from_str(""), None);
        assert_eq!(TestField::from_str("ID"), None);
        assert_eq!(TestField::from_str("Name"), None);
    }

    #[test]
    fn test_test_field_to_string() {
        assert_eq!(TestField::Id.to_string(), "id");
        assert_eq!(TestField::Name.to_string(), "name");
        assert_eq!(TestField::CreatedAt.to_string(), "created_at");
        assert_eq!(TestField::UpdatedAt.to_string(), "updated_at");
    }

    #[test]
    fn test_test_field_clone_and_partial_eq() {
        let field1 = TestField::Id;
        let field2 = field1.clone();

        assert_eq!(field1, field2);
        assert_ne!(field1, TestField::Name);
    }

    #[test]
    fn test_parse_order_by_string_complex_scenario() {
        let order_by_str = "+name,-created_at,+id,-updated_at";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 4);

        assert_eq!(result[0].field, TestField::Name);
        assert!(matches!(result[0].order, SortOrder::Asc));

        assert_eq!(result[1].field, TestField::CreatedAt);
        assert!(matches!(result[1].order, SortOrder::Desc));

        assert_eq!(result[2].field, TestField::Id);
        assert!(matches!(result[2].order, SortOrder::Asc));

        assert_eq!(result[3].field, TestField::UpdatedAt);
        assert!(matches!(result[3].order, SortOrder::Desc));
    }

    #[test]
    fn test_parse_order_by_string_case_sensitivity() {
        let order_by_str = "+ID,Name"; // Should not match due to case sensitivity
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_order_by_string_special_characters() {
        let order_by_str = "++id,--name"; // Invalid prefix patterns
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_order_by_string_mixed_valid_invalid() {
        let order_by_str = "+id,invalid_field,-name,another_invalid,created_at";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].field, TestField::Id);
        assert!(matches!(result[0].order, SortOrder::Asc));
        assert_eq!(result[1].field, TestField::Name);
        assert!(matches!(result[1].order, SortOrder::Desc));
        assert_eq!(result[2].field, TestField::CreatedAt);
        assert!(matches!(result[2].order, SortOrder::Asc));
    }

    #[test]
    fn test_sort_order_equality() {
        let asc1 = SortOrder::Asc;
        let asc2 = SortOrder::Asc;
        let desc1 = SortOrder::Desc;

        assert!(asc1 == asc2);
        assert!(asc1 != desc1);
    }

    #[test]
    fn test_order_by_field_round_trip() {
        let fields = [
            TestField::Id,
            TestField::Name,
            TestField::CreatedAt,
            TestField::UpdatedAt,
        ];

        for field in &fields {
            let string_repr = field.to_string();
            let parsed_field = TestField::from_str(&string_repr);
            assert_eq!(parsed_field, Some(field.clone()));
        }
    }
}
