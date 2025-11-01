use my_axum::{
    core::{context::Context, db::entity::user},
    pkg::password::hash_password_string,
    user::{repository::user_repository, service::auth_service},
};
use sea_orm::ActiveValue::Set;

pub async fn login_normal_user(context: &mut Context<'_>) -> (String, String) {
    let hashed_password = hash_password_string("user_password").await.unwrap();
    let user = user::ActiveModel {
        email: Set("user@example.com".to_string()),
        password: Set(hashed_password),
        first_name: Set(Some("Normal".to_string())),
        last_name: Set(Some("User".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let user = user_repository::create(context, user).await.unwrap();
    context.user = Some(user.clone());
    let (access_token, refresh_token) = auth_service::generate_token_pair(user.id).await.unwrap();
    (access_token, refresh_token)
}

#[allow(dead_code)]
pub async fn login_admin_user(context: &mut Context<'_>) -> (String, String) {
    let hashed_password = hash_password_string("admin_password").await.unwrap();
    let user = user::ActiveModel {
        email: Set("admin@example.com".to_string()),
        password: Set(hashed_password),
        first_name: Set(Some("Admin".to_string())),
        last_name: Set(Some("User".to_string())),
        phone: Set(None),
        ..Default::default()
    };
    let user = user_repository::create(context, user).await.unwrap();
    context.user = Some(user.clone());
    let (access_token, refresh_token) = auth_service::generate_token_pair(user.id).await.unwrap();
    (access_token, refresh_token)
}
