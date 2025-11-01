use crate::{
    config::setting::Setting,
    core::{
        context::Context,
        db::{connection::get_db, entity::user},
    },
    pkg::password::hash_password,
    user::repository::user_repository,
};

use sea_orm::{DbErr, TransactionTrait, entity::*};

pub async fn create_user_if_not_exists(
    context: &mut Context<'_>,
    email: &str,
    password: &str,
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
        first_name: Set(first_name.map(|s| s.to_string())),
        last_name: Set(last_name.map(|s| s.to_string())),
        phone: Set(phone.map(|s| s.to_string())),
        ..Default::default()
    };

    let inserted_user = user_repository::create(context, new_user).await?;

    println!("Created new user with email '{}'.", email);
    Ok(Some(inserted_user))
}

pub async fn seed_users(context: &mut Context<'_>) -> anyhow::Result<()> {
    println!("🌱 Seeding users...");

    let admin_user = create_user_if_not_exists(
        context,
        "admin@example.com",
        "admin123@",
        Some("Admin"),
        Some("User"),
        Some("+1987654321"),
    )
    .await?;

    // Set the admin user in context for created_user_id references
    context.user = admin_user;

    let _ = create_user_if_not_exists(
        context,
        "user@example.com",
        "password123@",
        Some("John"),
        Some("Doe"),
        Some("+1234567890"),
    )
    .await?;

    Ok(())
}

pub async fn run(setting: &Setting) -> anyhow::Result<()> {
    println!("🚀 Starting database seeding...");

    // Get database connection
    let db = get_db(&setting.database_url).await?;

    db.transaction::<_, (), DbErr>(|txn| {
        Box::pin(async move {
            let mut context = Context {
                txn,
                user: None,
                producer: None,
            };
            seed_users(&mut context)
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))
        })
    })
    .await?;

    println!("🎉 Database seeding completed successfully!");
    Ok(())
}
