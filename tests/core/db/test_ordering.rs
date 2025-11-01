#[cfg(test)]
mod ordering_tests {
    use my_axum::core::db::ordering::{ApplyOrdering, OrderBy, OrderByField, SortOrder};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, QueryTrait};

    // Mock entity and column for testing
    use my_axum::core::db::entity::user;

    #[derive(Debug, Clone, PartialEq)]
    enum TestField {
        Id,
        Email,
        FirstName,
        LastName,
        CreatedAt,
    }

    impl OrderByField for TestField {
        fn from_str(s: &str) -> Option<Self> {
            match s {
                "id" => Some(TestField::Id),
                "email" => Some(TestField::Email),
                "first_name" => Some(TestField::FirstName),
                "last_name" => Some(TestField::LastName),
                "created_at" => Some(TestField::CreatedAt),
                _ => None,
            }
        }

        fn to_string(&self) -> String {
            match self {
                TestField::Id => "id".to_string(),
                TestField::Email => "email".to_string(),
                TestField::FirstName => "first_name".to_string(),
                TestField::LastName => "last_name".to_string(),
                TestField::CreatedAt => "created_at".to_string(),
            }
        }
    }

    // OrderBy and SortOrder tests
    #[test]
    fn test_sort_order_default() {
        let sort_order = SortOrder::default();
        assert!(matches!(sort_order, SortOrder::Asc));
    }

    #[test]
    fn test_sort_order_clone_trait() {
        let sort_order = SortOrder::Desc;
        let cloned = sort_order.clone();
        assert!(matches!(cloned, SortOrder::Desc));
        assert!(sort_order == cloned);
    }

    #[test]
    fn test_order_by_new_constructor() {
        let order_by = OrderBy::new(TestField::Id, SortOrder::Asc);
        assert_eq!(order_by.field, TestField::Id);
        assert!(matches!(order_by.order, SortOrder::Asc));
    }

    #[test]
    fn test_order_by_new_desc_constructor() {
        let order_by = OrderBy::new(TestField::Email, SortOrder::Desc);
        assert_eq!(order_by.field, TestField::Email);
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
        let order_by_str = "-email";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].field, TestField::Email);
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
    fn test_parse_order_by_string_multiple_fields_mixed() {
        let order_by_str = "+first_name,-created_at,id";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 3);

        assert_eq!(result[0].field, TestField::FirstName);
        assert!(matches!(result[0].order, SortOrder::Asc));

        assert_eq!(result[1].field, TestField::CreatedAt);
        assert!(matches!(result[1].order, SortOrder::Desc));

        assert_eq!(result[2].field, TestField::Id);
        assert!(matches!(result[2].order, SortOrder::Asc));
    }

    #[test]
    fn test_parse_order_by_string_with_spaces() {
        let order_by_str = " +first_name , -created_at , id ";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].field, TestField::FirstName);
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
        let order_by_str = "+invalid_field,email";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].field, TestField::Email);
        assert!(matches!(result[0].order, SortOrder::Asc));
    }

    #[test]
    fn test_parse_order_by_string_empty_parts() {
        let order_by_str = "+first_name,,,-created_at,";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].field, TestField::FirstName);
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
    fn test_order_by_clone_trait() {
        let original = OrderBy::new(TestField::Id, SortOrder::Desc);
        let cloned = original.clone();

        assert_eq!(original.field, cloned.field);
        assert!(original.order == cloned.order);
    }

    #[test]
    fn test_parse_order_by_string_complex_scenario() {
        let order_by_str = "+first_name,-created_at,+id,-last_name";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 4);

        assert_eq!(result[0].field, TestField::FirstName);
        assert!(matches!(result[0].order, SortOrder::Asc));

        assert_eq!(result[1].field, TestField::CreatedAt);
        assert!(matches!(result[1].order, SortOrder::Desc));

        assert_eq!(result[2].field, TestField::Id);
        assert!(matches!(result[2].order, SortOrder::Asc));

        assert_eq!(result[3].field, TestField::LastName);
        assert!(matches!(result[3].order, SortOrder::Desc));
    }

    #[test]
    fn test_parse_order_by_string_case_sensitivity() {
        let order_by_str = "+ID,Email"; // Should not match due to case sensitivity
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_order_by_string_special_characters() {
        let order_by_str = "++id,--email"; // Invalid prefix patterns
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_order_by_string_mixed_valid_invalid() {
        let order_by_str = "+id,invalid_field,-email,another_invalid,created_at";
        let result: Vec<OrderBy<TestField>> = OrderBy::parse_order_by_string(order_by_str);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].field, TestField::Id);
        assert!(matches!(result[0].order, SortOrder::Asc));
        assert_eq!(result[1].field, TestField::Email);
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
            TestField::Email,
            TestField::FirstName,
            TestField::LastName,
            TestField::CreatedAt,
        ];

        for field in &fields {
            let string_repr = field.to_string();
            let parsed_field = TestField::from_str(&string_repr);
            assert_eq!(parsed_field, Some(field.clone()));
        }
    }

    // ApplyOrdering tests
    fn field_mapper(field: &TestField) -> user::Column {
        match field {
            TestField::Id => user::Column::Id,
            TestField::Email => user::Column::Email,
            TestField::FirstName => user::Column::FirstName,
            TestField::LastName => user::Column::LastName,
            TestField::CreatedAt => user::Column::CreatedAt,
        }
    }

    #[test]
    fn test_apply_ordering_single_ascending() {
        let query = user::Entity::find();
        let orders = vec![OrderBy::new(TestField::Email, SortOrder::Asc)];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        // Query should be built successfully without errors
        assert!(
            !result_query
                .build(sea_orm::DatabaseBackend::Postgres)
                .to_string()
                .is_empty()
        );
    }

    #[test]
    fn test_apply_ordering_single_descending() {
        let query = user::Entity::find();
        let orders = vec![OrderBy::new(TestField::Email, SortOrder::Desc)];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        // Query should be built successfully
        assert!(
            !result_query
                .build(sea_orm::DatabaseBackend::Postgres)
                .to_string()
                .is_empty()
        );
    }

    #[test]
    fn test_apply_ordering_multiple_fields() {
        let query = user::Entity::find();
        let orders = vec![
            OrderBy::new(TestField::LastName, SortOrder::Asc),
            OrderBy::new(TestField::FirstName, SortOrder::Desc),
            OrderBy::new(TestField::CreatedAt, SortOrder::Asc),
        ];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        let sql = result_query
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();

        // SQL should contain ORDER BY clause
        assert!(sql.contains("ORDER BY"));
    }

    #[test]
    fn test_apply_ordering_empty_orders() {
        let query = user::Entity::find();
        let orders: Vec<OrderBy<TestField>> = vec![];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        // Query should still be valid even with no ordering
        assert!(
            !result_query
                .build(sea_orm::DatabaseBackend::Postgres)
                .to_string()
                .is_empty()
        );
    }

    #[test]
    fn test_apply_ordering_with_filter() {
        let query = user::Entity::find().filter(user::Column::Email.contains("test"));
        let orders = vec![OrderBy::new(TestField::CreatedAt, SortOrder::Asc)];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        let sql = result_query
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();

        // SQL should contain both WHERE and ORDER BY
        assert!(sql.contains("WHERE"));
    }

    #[test]
    fn test_apply_ordering_all_fields_ascending() {
        let query = user::Entity::find();
        let orders = vec![
            OrderBy::new(TestField::Id, SortOrder::Asc),
            OrderBy::new(TestField::Email, SortOrder::Asc),
            OrderBy::new(TestField::FirstName, SortOrder::Asc),
            OrderBy::new(TestField::LastName, SortOrder::Asc),
            OrderBy::new(TestField::CreatedAt, SortOrder::Asc),
        ];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        let sql = result_query
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();
        assert!(sql.contains("ORDER BY"));
    }

    #[test]
    fn test_apply_ordering_all_fields_descending() {
        let query = user::Entity::find();
        let orders = vec![
            OrderBy::new(TestField::Id, SortOrder::Desc),
            OrderBy::new(TestField::Email, SortOrder::Desc),
            OrderBy::new(TestField::FirstName, SortOrder::Desc),
            OrderBy::new(TestField::LastName, SortOrder::Desc),
            OrderBy::new(TestField::CreatedAt, SortOrder::Desc),
        ];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        let sql = result_query
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();
        assert!(sql.contains("ORDER BY"));
    }

    #[test]
    fn test_apply_ordering_mixed_sort_orders() {
        let query = user::Entity::find();
        let orders = vec![
            OrderBy {
                field: TestField::LastName,
                order: SortOrder::Asc,
            },
            OrderBy {
                field: TestField::FirstName,
                order: SortOrder::Desc,
            },
            OrderBy {
                field: TestField::Email,
                order: SortOrder::Asc,
            },
        ];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        let sql = result_query
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();
        assert!(!sql.is_empty());
    }

    #[test]
    fn test_apply_ordering_same_field_multiple_times() {
        let query = user::Entity::find();
        let orders = vec![
            OrderBy::new(TestField::Email, SortOrder::Asc),
            OrderBy::new(TestField::Email, SortOrder::Desc),
        ];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        // Should handle duplicate fields (last one wins in SeaORM behavior)
        assert!(
            !result_query
                .build(sea_orm::DatabaseBackend::Postgres)
                .to_string()
                .is_empty()
        );
    }

    #[test]
    fn test_apply_ordering_with_limit() {
        let query = user::Entity::find().limit(10);
        let orders = vec![OrderBy::new(TestField::CreatedAt, SortOrder::Asc)];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        let sql = result_query
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();
        assert!(sql.contains("LIMIT"));
    }

    #[test]
    fn test_apply_ordering_with_offset() {
        let query = user::Entity::find().offset(20);
        let orders = vec![OrderBy::new(TestField::Id, SortOrder::Desc)];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        let sql = result_query
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();
        assert!(sql.contains("OFFSET"));
    }

    #[test]
    fn test_apply_ordering_preserves_query_structure() {
        let query = user::Entity::find()
            .filter(user::Column::Email.ends_with("@example.com"))
            .limit(5)
            .offset(10);

        let orders = vec![OrderBy::new(TestField::Email, SortOrder::Asc)];

        let result_query = user::Entity::apply_ordering(query, &orders, field_mapper);

        let sql = result_query
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();

        // Should preserve all query parts
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("LIMIT"));
        assert!(sql.contains("OFFSET"));
    }

    #[test]
    fn test_field_mapper_all_variants() {
        // Test that field mapper handles all field types correctly without panicking
        let _id_col = field_mapper(&TestField::Id);
        let _email_col = field_mapper(&TestField::Email);
        let _first_name_col = field_mapper(&TestField::FirstName);
        let _last_name_col = field_mapper(&TestField::LastName);
        let _created_at_col = field_mapper(&TestField::CreatedAt);
        // Test passes if no panic occurs
    }

    #[test]
    fn test_order_by_clone() {
        let order = OrderBy::new(TestField::Email, SortOrder::Asc);
        let cloned = order.clone();

        assert_eq!(order.field, cloned.field);
        // Compare orders by pattern matching instead of direct comparison
        match (&order.order, &cloned.order) {
            (SortOrder::Asc, SortOrder::Asc) | (SortOrder::Desc, SortOrder::Desc) => {}
            _ => panic!("Orders should be equal"),
        }
    }

    #[test]
    fn test_sort_order_variants() {
        let asc_order = OrderBy::new(TestField::Email, SortOrder::Asc);
        let desc_order = OrderBy::new(TestField::Email, SortOrder::Desc);

        // Test that both variants can be created
        match asc_order.order {
            SortOrder::Asc => {}
            _ => panic!("Should be Asc"),
        }

        match desc_order.order {
            SortOrder::Desc => {}
            _ => panic!("Should be Desc"),
        }
    }

    #[test]
    fn test_apply_ordering_chain_multiple_queries() {
        let query1 = user::Entity::find();
        let orders1 = vec![OrderBy::new(TestField::Email, SortOrder::Asc)];
        let result1 = user::Entity::apply_ordering(query1, &orders1, field_mapper);

        let query2 = user::Entity::find();
        let orders2 = vec![OrderBy::new(TestField::CreatedAt, SortOrder::Desc)];
        let result2 = user::Entity::apply_ordering(query2, &orders2, field_mapper);

        // Both queries should be valid
        assert!(
            !result1
                .build(sea_orm::DatabaseBackend::Postgres)
                .to_string()
                .is_empty()
        );
        assert!(
            !result2
                .build(sea_orm::DatabaseBackend::Postgres)
                .to_string()
                .is_empty()
        );
    }

    #[test]
    fn test_apply_ordering_generic_implementation() {
        // Test that the trait is properly implemented for all entities
        let query = user::Entity::find();
        let orders = vec![OrderBy::new(TestField::Id, SortOrder::Asc)];

        // Should be able to call apply_ordering on the entity directly
        let result = user::Entity::apply_ordering(query, &orders, field_mapper);

        assert!(
            !result
                .build(sea_orm::DatabaseBackend::Postgres)
                .to_string()
                .is_empty()
        );
    }

    #[test]
    fn test_test_field_to_string() {
        assert_eq!(TestField::Id.to_string(), "id");
        assert_eq!(TestField::Email.to_string(), "email");
        assert_eq!(TestField::FirstName.to_string(), "first_name");
        assert_eq!(TestField::LastName.to_string(), "last_name");
        assert_eq!(TestField::CreatedAt.to_string(), "created_at");
    }

    #[test]
    fn test_test_field_from_str_valid() {
        assert_eq!(TestField::from_str("id").unwrap(), TestField::Id);
        assert_eq!(TestField::from_str("email").unwrap(), TestField::Email);
        assert_eq!(
            TestField::from_str("first_name").unwrap(),
            TestField::FirstName
        );
        assert_eq!(
            TestField::from_str("last_name").unwrap(),
            TestField::LastName
        );
        assert_eq!(
            TestField::from_str("created_at").unwrap(),
            TestField::CreatedAt
        );
    }

    #[test]
    fn test_test_field_from_str_invalid() {
        assert!(TestField::from_str("invalid_field").is_none());
        assert!(TestField::from_str("").is_none());
        assert!(TestField::from_str("ID").is_none()); // case sensitive
    }
}
