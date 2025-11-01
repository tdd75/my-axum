use std::time::Duration;

pub async fn get_db(database_url: &str) -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
    let mut opt = sea_orm::ConnectOptions::new(database_url);

    opt.max_connections(16)
        .min_connections(8)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(60))
        .max_lifetime(Duration::from_secs(1800))
        .sqlx_logging(true);

    sea_orm::Database::connect(opt).await
}
