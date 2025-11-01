use sea_orm::{EntityTrait, QueryOrder, Select};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Generic sort order enum
#[derive(Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    #[default]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

/// Generic order by field trait
pub trait OrderByField: Clone + PartialEq + std::fmt::Debug {
    /// Parse a field name string into the specific field enum
    fn from_str(s: &str) -> Option<Self>;

    /// Get the string representation of the field
    fn to_string(&self) -> String;
}

/// Generic order by structure
#[derive(Clone)]
pub struct OrderBy<T: OrderByField> {
    pub field: T,
    pub order: SortOrder,
}

impl<T: OrderByField> OrderBy<T> {
    /// Create a new OrderBy instance
    pub fn new(field: T, order: SortOrder) -> Self {
        Self { field, order }
    }

    /// Parse order_by string into Vec<OrderBy<T>>
    /// Format: "+field_name,-field_name,field_name"
    /// Examples: "+created_at", "-id", "+name,-created_at"
    pub fn parse_order_by_string(order_by_str: &str) -> Vec<OrderBy<T>> {
        order_by_str
            .split(',')
            .filter_map(|part| {
                let part = part.trim();
                if part.is_empty() {
                    return None;
                }

                let (order, field_str) = if let Some(stripped) = part.strip_prefix('+') {
                    (SortOrder::Asc, stripped)
                } else if let Some(stripped) = part.strip_prefix('-') {
                    (SortOrder::Desc, stripped)
                } else {
                    (SortOrder::Asc, part)
                };

                T::from_str(field_str).map(|field| OrderBy { field, order })
            })
            .collect()
    }
}

/// Generic trait for applying ordering to SeaORM queries
pub trait ApplyOrdering<E: EntityTrait> {
    /// Apply ordering to a SeaORM Select query
    fn apply_ordering<T: OrderByField>(
        query: Select<E>,
        orders: &[OrderBy<T>],
        field_mapper: impl Fn(&T) -> E::Column,
    ) -> Select<E> {
        let mut query = query;

        for order_by in orders {
            let column = field_mapper(&order_by.field);
            match order_by.order {
                SortOrder::Asc => {
                    query = query.order_by_asc(column);
                }
                SortOrder::Desc => {
                    query = query.order_by_desc(column);
                }
            }
        }

        query
    }
}

// Implement for all entities
impl<E: EntityTrait> ApplyOrdering<E> for E {}

#[cfg(test)]
mod tests {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryTrait};

    use super::{ApplyOrdering, OrderBy, OrderByField, SortOrder};
    use crate::core::db::entity::user;

    #[derive(Debug, Clone, PartialEq)]
    enum TestField {
        Id,
        Email,
        CreatedAt,
    }

    impl OrderByField for TestField {
        fn from_str(s: &str) -> Option<Self> {
            match s {
                "id" => Some(Self::Id),
                "email" => Some(Self::Email),
                "created_at" => Some(Self::CreatedAt),
                _ => None,
            }
        }

        fn to_string(&self) -> String {
            match self {
                Self::Id => "id".to_string(),
                Self::Email => "email".to_string(),
                Self::CreatedAt => "created_at".to_string(),
            }
        }
    }

    fn field_mapper(field: &TestField) -> user::Column {
        match field {
            TestField::Id => user::Column::Id,
            TestField::Email => user::Column::Email,
            TestField::CreatedAt => user::Column::CreatedAt,
        }
    }

    #[test]
    fn parses_order_by_string() {
        let orders = OrderBy::<TestField>::parse_order_by_string("+email,-created_at,id");
        assert_eq!(orders.len(), 3);
        assert_eq!(orders[0].field, TestField::Email);
        assert!(matches!(orders[0].order, SortOrder::Asc));
        assert!(matches!(orders[1].order, SortOrder::Desc));
    }

    #[test]
    fn ignores_invalid_fields() {
        let orders = OrderBy::<TestField>::parse_order_by_string("invalid,+email");
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].field, TestField::Email);
    }

    #[test]
    fn applies_ordering_to_query() {
        let query = user::Entity::find().filter(user::Column::Email.contains("@example.com"));
        let orders = vec![
            OrderBy::new(TestField::Email, SortOrder::Asc),
            OrderBy::new(TestField::CreatedAt, SortOrder::Desc),
        ];

        let sql = user::Entity::apply_ordering(query, &orders, field_mapper)
            .build(sea_orm::DatabaseBackend::Postgres)
            .to_string();

        assert!(sql.contains("WHERE"));
        assert!(sql.contains("ORDER BY"));
    }
}
