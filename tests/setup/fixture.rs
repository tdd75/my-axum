use my_axum::{
    core::context::Context,
    pkg::password::hash_password_string,
    user::entity::{sea_orm_active_enums::UserRole, user},
    user::{repository::user_repository, service::auth_service},
};
use sea_orm::ActiveValue::Set;

pub async fn login_normal_user(context: &mut Context) -> (String, String) {
    let hashed_password = hash_password_string("user_password").await.unwrap();
    let user = user::ActiveModel {
        email: Set("user@example.com".to_string()),
        password: Set(hashed_password),
        role: Set(UserRole::User),
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
pub async fn login_admin_user(context: &mut Context) -> (String, String) {
    let hashed_password = hash_password_string("admin_password").await.unwrap();
    let user = user::ActiveModel {
        email: Set("admin@example.com".to_string()),
        password: Set(hashed_password),
        role: Set(UserRole::Admin),
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
