use crate::user::entity::user;
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

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{
        UserCreateDTO, UserDTO, UserListDTO, UserSearchParamsDTO, UserSimpleDTO, UserUpdateDTO,
        UserWithRelations,
    };
    use crate::user::entity::{sea_orm_active_enums::UserRole, user};

    fn sample_user(id: i32, email: &str) -> user::Model {
        let now = Utc::now().naive_utc();
        user::Model {
            id,
            email: email.to_string(),
            password: "password".to_string(),
            role: UserRole::User,
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            phone: Some("123456789".to_string()),
            created_at: Some(now),
            updated_at: Some(now),
            created_user_id: None,
            updated_user_id: None,
        }
    }

    #[test]
    fn converts_model_to_user_simple_dto() {
        let dto = UserSimpleDTO::from(sample_user(1, "simple@example.com"));
        assert_eq!(dto.id, 1);
        assert_eq!(dto.email, "simple@example.com");
    }

    #[test]
    fn converts_user_with_relations_to_user_dto() {
        let dto: UserDTO = UserWithRelations {
            model: sample_user(1, "user@example.com"),
            created_user: Some(sample_user(2, "creator@example.com")),
            updated_user: Some(sample_user(3, "updater@example.com")),
        }
        .into();

        assert_eq!(dto.id, 1);
        assert_eq!(dto.email, "user@example.com");
        assert_eq!(
            dto.created_user.as_ref().unwrap().email,
            "creator@example.com"
        );
        assert_eq!(
            dto.updated_user.as_ref().unwrap().email,
            "updater@example.com"
        );
    }

    #[test]
    fn creates_input_dtos() {
        let create_dto = UserCreateDTO {
            email: "create@example.com".to_string(),
            password: "password".to_string(),
            first_name: Some("Create".to_string()),
            last_name: None,
            phone: None,
        };
        let update_dto = UserUpdateDTO {
            email: Some("update@example.com".to_string()),
            password: None,
            first_name: None,
            last_name: Some("Updated".to_string()),
            phone: None,
        };
        let search_dto = UserSearchParamsDTO {
            email: Some("search@example.com".to_string()),
            first_name: None,
            last_name: None,
            page: Some(2),
            page_size: Some(20),
            order_by: Some("+created_at".to_string()),
        };

        assert_eq!(create_dto.email, "create@example.com");
        assert_eq!(update_dto.last_name.as_deref(), Some("Updated"));
        assert_eq!(search_dto.page, Some(2));
    }

    #[test]
    fn creates_user_list_dto() {
        let list = UserListDTO {
            items: vec![
                UserWithRelations {
                    model: sample_user(1, "first@example.com"),
                    created_user: None,
                    updated_user: None,
                }
                .into(),
                UserWithRelations {
                    model: sample_user(2, "second@example.com"),
                    created_user: None,
                    updated_user: None,
                }
                .into(),
            ],
            count: 2,
        };

        assert_eq!(list.count, 2);
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[0].email, "first@example.com");
    }
}
