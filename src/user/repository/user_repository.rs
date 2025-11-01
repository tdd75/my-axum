use sea_orm::{entity::*, query::*};

use crate::core::{
    context::Context,
    db::{
        entity::user,
        ordering::{ApplyOrdering, OrderBy, OrderByField},
        pagination::calculate_offset,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum UserOrderByField {
    Id,
    Email,
    FirstName,
    LastName,
    CreatedAt,
    UpdatedAt,
}

impl OrderByField for UserOrderByField {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "id" => Some(UserOrderByField::Id),
            "email" => Some(UserOrderByField::Email),
            "first_name" => Some(UserOrderByField::FirstName),
            "last_name" => Some(UserOrderByField::LastName),
            "created_at" => Some(UserOrderByField::CreatedAt),
            "updated_at" => Some(UserOrderByField::UpdatedAt),
            _ => None,
        }
    }

    fn to_string(&self) -> String {
        match self {
            UserOrderByField::Id => "id".to_string(),
            UserOrderByField::Email => "email".to_string(),
            UserOrderByField::FirstName => "first_name".to_string(),
            UserOrderByField::LastName => "last_name".to_string(),
            UserOrderByField::CreatedAt => "created_at".to_string(),
            UserOrderByField::UpdatedAt => "updated_at".to_string(),
        }
    }
}

pub type UserOrderBy = OrderBy<UserOrderByField>;

#[derive(Default)]
pub struct UserSearchParams<'a> {
    pub ids: Option<&'a [i32]>,
    pub email: Option<&'a str>,
    pub first_name: Option<&'a str>,
    pub last_name: Option<&'a str>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub order_by: Option<&'a [UserOrderBy]>,
}

#[allow(clippy::too_many_arguments)]
pub async fn search(
    context: &Context<'_>,
    params: &UserSearchParams<'_>,
) -> Result<Vec<user::Model>, sea_orm::DbErr> {
    let mut query = user::Entity::find();

    // Apply filters
    if let Some(ids) = params.ids {
        query = query.filter(user::Column::Id.is_in(ids.to_vec()));
    }
    if let Some(email) = params.email {
        query = query.filter(user::Column::Email.contains(email));
    }
    if let Some(first_name) = params.first_name {
        query = query.filter(user::Column::FirstName.contains(first_name));
    }
    if let Some(last_name) = params.last_name {
        query = query.filter(user::Column::LastName.contains(last_name));
    }

    // Apply ordering using generic system
    if let Some(orders) = params.order_by {
        query = user::Entity::apply_ordering(query, orders, |field| match field {
            UserOrderByField::Id => user::Column::Id,
            UserOrderByField::Email => user::Column::Email,
            UserOrderByField::FirstName => user::Column::FirstName,
            UserOrderByField::LastName => user::Column::LastName,
            UserOrderByField::CreatedAt => user::Column::CreatedAt,
            UserOrderByField::UpdatedAt => user::Column::UpdatedAt,
        });
    }

    // Apply pagination
    if let Some(page_size) = params.page_size {
        let offset = calculate_offset(params.page, page_size);
        query = query.limit(page_size).offset(offset);
    }

    query.all(context.txn).await
}

pub async fn find_by_id(
    context: &Context<'_>,
    id: i32,
) -> Result<Option<user::Model>, sea_orm::DbErr> {
    user::Entity::find_by_id(id).one(context.txn).await
}

pub async fn find_by_email(
    context: &Context<'_>,
    email: &str,
) -> Result<Option<user::Model>, sea_orm::DbErr> {
    user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(context.txn)
        .await
}

pub async fn create(
    context: &Context<'_>,
    mut user: user::ActiveModel,
) -> Result<user::Model, sea_orm::DbErr> {
    user.created_at = Set(Some(chrono::Utc::now().naive_utc()));
    user.updated_at = Set(Some(chrono::Utc::now().naive_utc()));
    user.created_user_id = Set(context.user.as_ref().map(|u| u.id));
    user.updated_user_id = Set(context.user.as_ref().map(|u| u.id));

    user.insert(context.txn).await
}

pub async fn update(
    context: &Context<'_>,
    mut user: user::ActiveModel,
) -> Result<user::Model, sea_orm::DbErr> {
    user.updated_at = Set(Some(chrono::Utc::now().naive_utc()));
    user.updated_user_id = Set(context.user.as_ref().map(|u| u.id));

    user.update(context.txn).await
}

pub async fn delete(context: &Context<'_>, user: user::ActiveModel) -> Result<(), sea_orm::DbErr> {
    user.delete(context.txn).await?;

    Ok(())
}

pub async fn delete_by_id(context: &Context<'_>, id: i32) -> Result<(), sea_orm::DbErr> {
    user::Entity::delete_by_id(id).exec(context.txn).await?;

    Ok(())
}
