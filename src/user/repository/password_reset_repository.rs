use chrono::Utc;
use sea_orm::{DbErr, entity::*, query::*};

use crate::core::{context::Context, db::entity::password_reset_token};

pub async fn find_by_token(
    context: &Context<'_>,
    otp: &str,
) -> Result<Option<password_reset_token::Model>, DbErr> {
    password_reset_token::Entity::find()
        .filter(password_reset_token::Column::Token.eq(otp))
        .one(context.txn)
        .await
}

pub async fn create(
    context: &Context<'_>,
    mut reset_token: password_reset_token::ActiveModel,
) -> Result<password_reset_token::Model, sea_orm::DbErr> {
    reset_token.created_at = Set(Some(chrono::Utc::now().naive_utc()));

    reset_token.insert(context.txn).await
}

pub async fn update(
    context: &Context<'_>,
    reset_token: password_reset_token::ActiveModel,
) -> Result<password_reset_token::Model, sea_orm::DbErr> {
    reset_token.update(context.txn).await
}

pub async fn delete(
    context: &Context<'_>,
    reset_token: password_reset_token::ActiveModel,
) -> Result<(), sea_orm::DbErr> {
    reset_token.delete(context.txn).await?;
    Ok(())
}

pub async fn delete_expired(context: &Context<'_>) -> Result<(), sea_orm::DbErr> {
    let now = Utc::now().naive_utc();
    password_reset_token::Entity::delete_many()
        .filter(password_reset_token::Column::ExpiresAt.lt(now))
        .exec(context.txn)
        .await?;
    Ok(())
}

pub async fn delete_by_user_id(context: &Context<'_>, user_id: i32) -> Result<(), sea_orm::DbErr> {
    password_reset_token::Entity::delete_many()
        .filter(password_reset_token::Column::UserId.eq(user_id))
        .exec(context.txn)
        .await?;
    Ok(())
}
