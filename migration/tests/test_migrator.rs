use migration::Migrator;
use sea_orm_migration::{prelude::*, sea_orm::Database};

#[tokio::test]
async fn test_migrator_applies_all_migrations_and_builds_expected_schema() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let manager = SchemaManager::new(&db);

    let pending = Migrator::get_pending_migrations(&db).await.unwrap();
    let names = pending
        .iter()
        .map(|migration| migration.name().to_string())
        .collect::<Vec<_>>();

    assert_eq!(pending.len(), 4);
    assert_eq!(
        names,
        vec![
            "m20220101_000001_create_table",
            "m20251108_000002_add_refresh_token_table",
            "m20251130_000003_add_password_reset_token_table",
            "m20260412_000004_add_user_role",
        ]
    );

    Migrator::up(&db, None).await.unwrap();

    let applied = Migrator::get_applied_migrations(&db).await.unwrap();
    assert_eq!(applied.len(), 4);
    assert!(manager.has_table("user").await.unwrap());
    assert!(manager.has_table("refresh_token").await.unwrap());
    assert!(manager.has_table("password_reset_token").await.unwrap());
    assert!(manager.has_column("user", "role").await.unwrap());
}

#[tokio::test]
async fn test_migrator_rolls_back_all_migrations() {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let manager = SchemaManager::new(&db);

    Migrator::up(&db, None).await.unwrap();
    Migrator::down(&db, None).await.unwrap();

    assert!(!manager.has_table("user").await.unwrap());
    assert!(!manager.has_table("refresh_token").await.unwrap());
    assert!(!manager.has_table("password_reset_token").await.unwrap());

    let pending = Migrator::get_pending_migrations(&db).await.unwrap();
    assert_eq!(pending.len(), 4);
}
