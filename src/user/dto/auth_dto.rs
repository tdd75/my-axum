use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::core::db::entity::user;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginDTO {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterDTO {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct TokenPairDTO {
    pub access: String,
    pub refresh: String,
}

#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct RefreshTokenDTO {
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordDTO {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ForgotPasswordDTO {
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordDTO {
    pub email: String,
    pub otp: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProfileDTO {
    pub id: i32,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl From<user::Model> for ProfileDTO {
    fn from(model: user::Model) -> Self {
        ProfileDTO {
            id: model.id,
            email: model.email,
            first_name: model.first_name,
            last_name: model.last_name,
            phone: model.phone,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileDTO {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}
