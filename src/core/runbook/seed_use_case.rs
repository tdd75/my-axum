use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{TransactionTrait, entity::*};

use crate::{
    config::setting::Setting,
    core::{context::Context, db::connection::get_db},
    pkg::password::hash_password,
    user::{
        entity::{sea_orm_active_enums::UserRole, user},
        repository::user_repository,
    },
};

use super::{Runbook, RunbookError, RunbookExecutionResult, RunbookMetadata};

pub struct Seed;

#[async_trait]
impl Runbook for Seed {
    fn metadata(&self) -> RunbookMetadata {
        RunbookMetadata {
            name: "seed",
            description: "Seed default application data",
            usage: "runbook run seed",
        }
    }

    async fn run(
        &self,
        setting: &Setting,
        args: &[String],
    ) -> Result<RunbookExecutionResult, RunbookError> {
        if !args.is_empty() {
            return Err(RunbookError::bad_request(
                "seed does not accept any arguments",
            ));
        }

        seed(setting).await.map_err(RunbookError::internal_error)?;

        Ok(RunbookExecutionResult::new(
            "seed",
            "Seeded default application data",
        ))
    }
}

async fn seed(setting: &Setting) -> anyhow::Result<()> {
    println!("🚀 Starting database seeding...");

    let db = get_db(&setting.database_url).await?;

    let txn = db.begin().await?;
    let txn = Arc::new(txn);
    let mut context = Context::builder(txn.clone()).build();

    match seed_default_data(&mut context).await {
        Ok(()) => {
            drop(context);
            Arc::try_unwrap(txn)
                .map_err(|_| anyhow::anyhow!("Failed to unwrap transaction for commit"))?
                .commit()
                .await?;
        }
        Err(e) => {
            return Err(e);
        }
    }

    println!("🎉 Database seeding completed successfully!");
    Ok(())
}

async fn seed_default_data(context: &mut Context) -> anyhow::Result<()> {
    println!("🌱 Seeding default data...");

    let admin_user = create_user_if_not_exists(
        context,
        "admin@example.com",
        "admin123@",
        UserRole::Admin,
        Some("Admin"),
        Some("User"),
        Some("+1987654321"),
    )
    .await?;

    context.user = admin_user;

    let _ = create_user_if_not_exists(
        context,
        "user@example.com",
        "password123@",
        UserRole::User,
        Some("John"),
        Some("Doe"),
        Some("+1234567890"),
    )
    .await?;

    Ok(())
}

async fn create_user_if_not_exists(
    context: &mut Context,
    email: &str,
    password: &str,
    role: UserRole,
    first_name: Option<&str>,
    last_name: Option<&str>,
    phone: Option<&str>,
) -> Result<Option<user::Model>, anyhow::Error> {
    let existing_user = user_repository::find_by_email(context, email).await?;

    if let Some(user) = existing_user {
        println!(
            "User with email '{}' already exists. Skipping creation.",
            email
        );
        return Ok(Some(user));
    }

    let hashed_password = hash_password(password).await?;

    let new_user = user::ActiveModel {
        email: Set(email.to_string()),
        password: Set(hashed_password),
        role: Set(role),
        first_name: Set(first_name.map(|s| s.to_string())),
        last_name: Set(last_name.map(|s| s.to_string())),
        phone: Set(phone.map(|s| s.to_string())),
        ..Default::default()
    };

    let inserted_user = user_repository::create(context, new_user).await?;

    println!("Created new user with email '{}'.", email);
    Ok(Some(inserted_user))
}
