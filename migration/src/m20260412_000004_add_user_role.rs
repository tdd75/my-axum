use sea_orm_migration::prelude::{sea_query::extension::postgres::Type, *};
use sea_orm_migration::sea_orm::{ConnectionTrait, DbBackend};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        if db.get_database_backend() == DbBackend::Postgres {
            manager
                .create_type(
                    Type::create()
                        .as_enum(UserRole::Enum)
                        .values([UserRole::User, UserRole::Admin])
                        .to_owned(),
                )
                .await?;
        }

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::Role)
                            .enumeration(UserRole::Enum, [UserRole::User, UserRole::Admin])
                            .not_null()
                            .default("user"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::Role)
                    .to_owned(),
            )
            .await?;

        if db.get_database_backend() == DbBackend::Postgres {
            manager
                .drop_type(Type::drop().name(UserRole::Enum).to_owned())
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Role,
}

#[derive(DeriveIden)]
enum UserRole {
    #[sea_orm(iden = "user_role")]
    Enum,
    #[sea_orm(iden = "user")]
    User,
    #[sea_orm(iden = "admin")]
    Admin,
}
