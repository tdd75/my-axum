use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut foreign_key = ForeignKey::create()
            .name("fk-password_reset_token-user_id")
            .from(PasswordResetToken::Table, PasswordResetToken::UserId)
            .to(User::Table, User::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::NoAction)
            .to_owned();

        manager
            .create_table(
                Table::create()
                    .table(PasswordResetToken::Table)
                    .if_not_exists()
                    .col(pk_auto(PasswordResetToken::Id))
                    .col(integer(PasswordResetToken::UserId).not_null())
                    .col(
                        string_len(PasswordResetToken::Token, 6)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        integer(PasswordResetToken::RetryCount)
                            .not_null()
                            .default(0),
                    )
                    .col(timestamp(PasswordResetToken::ExpiresAt).not_null())
                    .col(timestamp_null(PasswordResetToken::CreatedAt))
                    .foreign_key(&mut foreign_key)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("ix_password_reset_token_user_id")
                    .table(PasswordResetToken::Table)
                    .col(PasswordResetToken::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("ix_password_reset_token_expires_at")
                    .table(PasswordResetToken::Table)
                    .col(PasswordResetToken::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PasswordResetToken::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PasswordResetToken {
    Table,
    Id,
    UserId,
    Token,
    RetryCount,
    ExpiresAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
