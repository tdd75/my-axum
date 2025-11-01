use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id))
                    .col(string(User::Email).unique_key())
                    .col(string(User::Password))
                    .col(string_null(User::FirstName))
                    .col(string_null(User::LastName))
                    .col(string_null(User::Phone))
                    .col(timestamp_null(User::CreatedAt))
                    .col(timestamp_null(User::UpdatedAt))
                    .col(integer_null(User::CreatedUserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_created_user")
                            .from(User::Table, User::CreatedUserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .col(integer_null(User::UpdatedUserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_updated_user")
                            .from(User::Table, User::UpdatedUserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Email,
    Password,
    FirstName,
    LastName,
    Phone,
    CreatedAt,
    UpdatedAt,
    CreatedUserId,
    UpdatedUserId,
}
