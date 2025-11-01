use dotenvy::dotenv;
use my_axum::{
    config::{app::App, setting::Setting},
    core::db::{connection::get_db, entity::prelude::*},
};
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Schema,
    TransactionTrait, sea_query::TableCreateStatement,
};
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub enum DatabaseType {
    Postgres,
    Sqlite,
}

pub struct TestApp {
    pub base_url: String,
    pub db: DatabaseConnection,
    pub db_url: String,
    pub setting: Setting,
}

impl TestApp {
    pub async fn spawn_app() -> Self {
        Self::spawn_app_with_db(DatabaseType::Sqlite).await
    }

    pub async fn spawn_db_only() -> Self {
        let _ = dotenv();

        let test_db_name = Self::random_db_name().await;
        let test_db_url = Self::get_sqlite_memory_url(&test_db_name);
        let db = Self::connect_sqlite_memory_db(&test_db_url).await.unwrap();
        Self::create_schema_from_entities(&db).await.unwrap();

        let mut setting = Setting::new();
        setting.database_url = test_db_url.clone();
        setting.messaging.message_broker = None;

        Self {
            base_url: String::new(),
            db,
            db_url: test_db_url,
            setting,
        }
    }

    pub async fn begin_transaction(&self) -> sea_orm::DatabaseTransaction {
        self.db.begin().await.unwrap()
    }

    pub async fn create_schema_from_entities(
        db: &DatabaseConnection,
    ) -> Result<(), sea_orm::DbErr> {
        let builder = db.get_database_backend();
        let schema = Schema::new(builder);

        if builder == DbBackend::Postgres {
            for create_enum in schema.create_enum_from_entity(User) {
                let stmt = create_enum.to_string(sea_orm::sea_query::PostgresQueryBuilder);
                db.execute_unprepared(&stmt).await?;
            }
        }

        // Create tables for all entities
        let entities: Vec<TableCreateStatement> = vec![
            schema.create_table_from_entity(User),
            schema.create_table_from_entity(RefreshToken),
            schema.create_table_from_entity(PasswordResetToken),
        ];

        for create_statement in entities {
            let stmt = match builder {
                DbBackend::Postgres => {
                    create_statement.to_string(sea_orm::sea_query::PostgresQueryBuilder)
                }
                DbBackend::MySql => {
                    create_statement.to_string(sea_orm::sea_query::MysqlQueryBuilder)
                }
                DbBackend::Sqlite => {
                    create_statement.to_string(sea_orm::sea_query::SqliteQueryBuilder)
                }
                _ => panic!("Unsupported database backend"),
            };
            db.execute_unprepared(&stmt).await?;
        }

        Ok(())
    }

    pub fn create_app_state(&self) -> my_axum::config::app::AppState {
        my_axum::config::app::AppState {
            db: self.db.clone(),
            setting: self.setting.clone(),
            producer: None,
        }
    }

    /// Compute database type from URL
    pub fn get_db_type(&self) -> DatabaseType {
        if self.db_url.starts_with("sqlite:") {
            DatabaseType::Sqlite
        } else if self.db_url.starts_with("postgres://") || self.db_url.starts_with("postgresql://")
        {
            DatabaseType::Postgres
        } else {
            panic!("Unsupported database URL format: {}", self.db_url)
        }
    }

    /// Extract database name from URL
    pub fn get_db_name(&self) -> Option<String> {
        match self.get_db_type() {
            DatabaseType::Sqlite => {
                // Extract file path from sqlite:path/to/file.db
                self.db_url
                    .strip_prefix("sqlite:")
                    .map(|path| path.to_string())
            }
            DatabaseType::Postgres => {
                // Extract database name from postgres://user:pass@host:port/dbname
                if let Some(last_slash) = self.db_url.rfind('/') {
                    let db_name = &self.db_url[last_slash + 1..];
                    if !db_name.is_empty() {
                        Some(db_name.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    pub async fn spawn_app_with_db(db_type: DatabaseType) -> Self {
        match db_type {
            DatabaseType::Sqlite => Self::spawn_app_sqlite().await,
            DatabaseType::Postgres => Self::spawn_app_postgres().await,
        }
    }

    async fn random_db_name() -> String {
        format!("test_db_{}", Uuid::new_v4().to_string().replace("-", "_"))
    }

    async fn create_and_run_app(test_db_url: String) -> (String, DatabaseConnection) {
        // Build the app
        let mut setting = Setting::new();
        setting.database_url = test_db_url;
        setting.app_port = 0;
        setting.messaging.message_broker = None; // Disable message broker for tests
        let app = App::new(setting).await.unwrap();

        let base_url = app.base_url.clone();
        let db = app.app_state.db.clone();

        // Spawn the app in the background
        tokio::spawn(app.run_until_stopped());

        (base_url, db)
    }

    async fn create_and_run_app_with_db(
        test_db_url: String,
        db: DatabaseConnection,
    ) -> (String, DatabaseConnection) {
        let mut setting = Setting::new();
        setting.database_url = test_db_url;
        setting.app_port = 0;
        setting.messaging.message_broker = None;

        let app = App::new_with_db(setting, db.clone()).await.unwrap();
        let base_url = app.base_url.clone();

        tokio::spawn(app.run_until_stopped());

        (base_url, db)
    }

    async fn connect_sqlite_memory_db(
        database_url: &str,
    ) -> Result<DatabaseConnection, sea_orm::DbErr> {
        let mut opt = ConnectOptions::new(database_url);

        opt.max_connections(1)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(None::<Duration>)
            .max_lifetime(None::<Duration>)
            .sqlx_logging(true);

        Database::connect(opt).await
    }

    async fn spawn_app_sqlite() -> Self {
        let _ = dotenv();

        let test_db_name = Self::random_db_name().await;
        let test_db_url = Self::get_sqlite_memory_url(&test_db_name);
        let db = Self::connect_sqlite_memory_db(&test_db_url).await.unwrap();
        Self::create_schema_from_entities(&db).await.unwrap();

        let (base_url, db) = Self::create_and_run_app_with_db(test_db_url.clone(), db).await;

        let mut setting = Setting::new();
        setting.database_url = test_db_url.clone();

        Self {
            base_url,
            db,
            db_url: test_db_url,
            setting,
        }
    }

    fn get_sqlite_memory_url(test_db_name: &str) -> String {
        format!("sqlite:file:{test_db_name}?mode=memory&cache=shared")
    }

    /// Spawn test app with PostgreSQL (dedicated test database)
    async fn spawn_app_postgres() -> Self {
        let _ = dotenv();

        let test_db_name = Self::random_db_name().await;
        let test_db_url = Self::setup_postgres_database(&test_db_name).await;

        let (base_url, db) = Self::create_and_run_app(test_db_url.clone()).await;

        let mut setting = Setting::new();
        setting.database_url = test_db_url.clone();

        Self {
            base_url,
            db,
            db_url: test_db_url,
            setting,
        }
    }

    fn get_postgres_admin_url() -> String {
        let setting = Setting::new();
        // Parse database URL and replace database name with 'postgres'
        let db_url = &setting.database_url;
        if let Some(last_slash) = db_url.rfind('/') {
            format!("{}/postgres", &db_url[..last_slash])
        } else {
            panic!("Invalid DATABASE_URL format: {}", db_url)
        }
    }

    fn get_postgres_test_url(test_db_name: &str) -> String {
        let setting = Setting::new();
        // Parse database URL and replace database name with test database name
        let db_url = &setting.database_url;
        if let Some(last_slash) = db_url.rfind('/') {
            format!("{}/{}", &db_url[..last_slash], test_db_name)
        } else {
            panic!("Invalid DATABASE_URL format: {}", db_url)
        }
    }
    async fn setup_postgres_database(test_db_name: &str) -> String {
        // Connect to postgres without selecting a database
        let admin_url = Self::get_postgres_admin_url();
        let db = get_db(&admin_url).await.unwrap();

        // Create test database if not exists
        let create_db_sql = format!("CREATE DATABASE {} WITH ENCODING 'UTF8'", test_db_name);
        db.execute_unprepared(&create_db_sql).await.ok();

        // Close connection to postgres
        db.close().await.ok();

        // Connect to the newly created test database
        let test_db_url = Self::get_postgres_test_url(test_db_name);

        let test_db = get_db(&test_db_url).await.unwrap();

        // Create schema from entities instead of running migrations
        Self::create_schema_from_entities(&test_db).await.unwrap();

        test_db_url
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        match self.get_db_type() {
            DatabaseType::Postgres => {
                if let Some(db_name) = self.get_db_name() {
                    let db_name = db_name.clone();
                    // Spawn a new thread with its own runtime to handle async cleanup
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async move {
                            // Connect to postgres to drop the test database
                            let admin_url = TestApp::get_postgres_admin_url();
                            if let Ok(db) = get_db(&admin_url).await {
                                // Now drop the database
                                let drop_db_sql =
                                    format!("DROP DATABASE IF EXISTS {} WITH (FORCE)", db_name);
                                db.execute_unprepared(&drop_db_sql).await.ok();
                                db.close().await.ok();
                            }
                        });
                    })
                    .join()
                    .ok();
                }
            }
            DatabaseType::Sqlite => {}
        }
    }
}
