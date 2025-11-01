use chrono::Utc;
use sea_orm::{entity::*, query::*};

use crate::core::{
    context::Context,
    db::{entity::refresh_token, pagination::calculate_offset},
};

#[derive(Default)]
pub struct RefreshTokenSearchParams<'a> {
    pub ids: Option<&'a [i32]>,
    pub user_id: Option<i32>,
    pub token: Option<&'a str>,
    pub is_expired: Option<bool>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[allow(clippy::too_many_arguments)]
pub async fn search(
    context: &Context<'_>,
    params: &RefreshTokenSearchParams<'_>,
) -> Result<Vec<refresh_token::Model>, sea_orm::DbErr> {
    let mut query = refresh_token::Entity::find();
    let now = Utc::now().naive_utc();

    // Apply filters
    if let Some(ids) = params.ids {
        query = query.filter(refresh_token::Column::Id.is_in(ids.to_vec()));
    }
    if let Some(user_id) = params.user_id {
        query = query.filter(refresh_token::Column::UserId.eq(user_id));
    }
    if let Some(token) = params.token {
        query = query.filter(refresh_token::Column::Token.contains(token));
    }
    match params.is_expired {
        Some(true) => {
            query = query.filter(refresh_token::Column::ExpiresAt.lte(now));
        }
        Some(false) => {
            query = query.filter(refresh_token::Column::ExpiresAt.gt(now));
        }
        None => {}
    }

    // Apply pagination
    if let Some(page_size) = params.page_size {
        let offset = calculate_offset(params.page, page_size);
        query = query.limit(page_size).offset(offset);
    }

    query.all(context.txn).await
}

pub async fn find_by_token(
    context: &Context<'_>,
    token: &str,
) -> Result<Option<refresh_token::Model>, sea_orm::DbErr> {
    refresh_token::Entity::find()
        .filter(refresh_token::Column::Token.eq(token))
        .one(context.txn)
        .await
}

pub async fn find_by_user_and_token(
    context: &Context<'_>,
    user_id: i32,
    token: &str,
) -> Result<Option<refresh_token::Model>, sea_orm::DbErr> {
    let now = Utc::now().naive_utc();
    refresh_token::Entity::find()
        .filter(refresh_token::Column::UserId.eq(user_id))
        .filter(refresh_token::Column::Token.eq(token))
        .filter(refresh_token::Column::ExpiresAt.gt(now))
        .one(context.txn)
        .await
}

pub async fn create(
    context: &Context<'_>,
    refresh_token: refresh_token::ActiveModel,
) -> Result<refresh_token::Model, sea_orm::DbErr> {
    refresh_token.insert(context.txn).await
}

pub async fn delete_by_token(context: &Context<'_>, token: &str) -> Result<(), sea_orm::DbErr> {
    refresh_token::Entity::delete_many()
        .filter(refresh_token::Column::Token.eq(token))
        .exec(context.txn)
        .await?;
    Ok(())
}

pub async fn delete_by_tokens(
    context: &Context<'_>,
    tokens: &[String],
) -> Result<(), sea_orm::DbErr> {
    refresh_token::Entity::delete_many()
        .filter(refresh_token::Column::Token.is_in(tokens.to_vec()))
        .exec(context.txn)
        .await?;
    Ok(())
}
