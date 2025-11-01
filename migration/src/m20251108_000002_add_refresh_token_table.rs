use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut foreign_key = ForeignKey::create()
            .name("fk-refresh_token-user_id")
            .from(RefreshToken::Table, RefreshToken::UserId)
            .to(User::Table, User::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::NoAction)
            .to_owned();

        manager
            .create_table(
                Table::create()
                    .table(RefreshToken::Table)
                    .if_not_exists()
                    .col(pk_auto(RefreshToken::Id))
                    .col(integer(RefreshToken::UserId).not_null())
                    .col(string_len(RefreshToken::Token, 512).not_null().unique_key())
                    .col(string_len_null(RefreshToken::DeviceInfo, 512))
                    .col(string_len_null(RefreshToken::IpAddress, 45))
                    .col(timestamp(RefreshToken::ExpiresAt).not_null())
                    .col(timestamp_null(RefreshToken::CreatedAt))
                    .foreign_key(&mut foreign_key)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("ix_refresh_token_user_id")
                    .table(RefreshToken::Table)
                    .col(RefreshToken::UserId)
                    .col(RefreshToken::Token)
                    .col(RefreshToken::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RefreshToken::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RefreshToken {
    Table,
    Id,
    UserId,
    Token,
    DeviceInfo,
    IpAddress,
    ExpiresAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}
