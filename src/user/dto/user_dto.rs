use crate::core::db::entity::user;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserSimpleDTO {
    pub id: i32,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl From<user::Model> for UserSimpleDTO {
    fn from(model: user::Model) -> Self {
        UserSimpleDTO {
            id: model.id,
            email: model.email,
            first_name: model.first_name,
            last_name: model.last_name,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserDTO {
    pub id: i32,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub created_user: Option<UserSimpleDTO>,
    pub updated_user: Option<UserSimpleDTO>,
}

pub struct UserWithRelations {
    pub model: user::Model,
    pub created_user: Option<user::Model>,
    pub updated_user: Option<user::Model>,
}

impl From<UserWithRelations> for UserDTO {
    fn from(input: UserWithRelations) -> Self {
        UserDTO {
            id: input.model.id,
            email: input.model.email,
            first_name: input.model.first_name,
            last_name: input.model.last_name,
            phone: input.model.phone,
            created_at: input.model.created_at,
            updated_at: input.model.updated_at,
            created_user: input.created_user.map(UserSimpleDTO::from),
            updated_user: input.updated_user.map(UserSimpleDTO::from),
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct UserSearchParamsDTO {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[param(default = 1)]
    pub page: Option<u64>,
    #[param(default = 10)]
    pub page_size: Option<u64>,
    pub order_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserListDTO {
    pub items: Vec<UserDTO>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserCreateDTO {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserUpdateDTO {
    pub email: Option<String>,
    pub password: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}
