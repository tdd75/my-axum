use my_axum::core::db::entity::user;
use my_axum::user::dto::user_dto::{
    UserCreateDTO, UserDTO, UserListDTO, UserSearchParamsDTO, UserUpdateDTO,
};

mod dto_tests {
    use my_axum::user::dto::user_dto::UserWithRelations;

    use super::*;

    #[tokio::test]
    async fn test_user_dto_from_model_conversion() {
        // Create a user model
        let now = chrono::Utc::now().naive_utc();

        let user_model = user::Model {
            id: 1,
            email: "test@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            phone: Some("123-456-7890".to_string()),
            created_at: Some(now),
            updated_at: Some(now),
            created_user_id: Some(1),
            updated_user_id: Some(2),
        };

        // Create related user models for testing
        let created_user_model = user::Model {
            id: 1,
            email: "creator@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Creator".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
            created_at: Some(now),
            updated_at: Some(now),
            created_user_id: None,
            updated_user_id: None,
        };

        let updated_user_model = user::Model {
            id: 2,
            email: "updater@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Updater".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
            created_at: Some(now),
            updated_at: Some(now),
            created_user_id: None,
            updated_user_id: None,
        };

        // Convert to DTO
        let user_dto: UserDTO = UserWithRelations {
            model: user_model,
            created_user: Some(created_user_model),
            updated_user: Some(updated_user_model),
        }
        .into();

        // Verify conversion
        assert_eq!(user_dto.id, 1);
        assert_eq!(user_dto.email, "test@example.com");
        assert_eq!(user_dto.first_name, Some("John".to_string()));
        assert_eq!(user_dto.last_name, Some("Doe".to_string()));
        assert_eq!(user_dto.phone, Some("123-456-7890".to_string()));
        assert_eq!(user_dto.created_at, Some(now));
        assert_eq!(user_dto.updated_at, Some(now));
        assert_eq!(user_dto.created_user.as_ref().unwrap().id, 1);
        assert_eq!(user_dto.updated_user.as_ref().unwrap().id, 2);
    }

    #[tokio::test]
    async fn test_user_dto_from_model_with_none_values() {
        let user_model = user::Model {
            id: 2,
            email: "minimal@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: None,
            last_name: None,
            phone: None,
            created_at: None,
            updated_at: None,
            created_user_id: None,
            updated_user_id: None,
        };

        let user_dto: UserDTO = UserWithRelations {
            model: user_model,
            created_user: None,
            updated_user: None,
        }
        .into();

        assert_eq!(user_dto.id, 2);
        assert_eq!(user_dto.email, "minimal@example.com");
        assert_eq!(user_dto.first_name, None);
        assert_eq!(user_dto.last_name, None);
        assert_eq!(user_dto.phone, None);
        assert_eq!(user_dto.created_at, None);
        assert_eq!(user_dto.updated_at, None);
        assert!(user_dto.created_user.is_none());
        assert!(user_dto.updated_user.is_none());
    }

    #[tokio::test]
    async fn test_user_create_dto_creation() {
        let create_dto = UserCreateDTO {
            email: "create@example.com".to_string(),
            password: "secure_password".to_string(),
            first_name: Some("Create".to_string()),
            last_name: Some("Test".to_string()),
            phone: Some("555-0123".to_string()),
        };

        assert_eq!(create_dto.email, "create@example.com");
        assert_eq!(create_dto.password, "secure_password");
        assert_eq!(create_dto.first_name, Some("Create".to_string()));
        assert_eq!(create_dto.last_name, Some("Test".to_string()));
        assert_eq!(create_dto.phone, Some("555-0123".to_string()));
    }

    #[tokio::test]
    async fn test_user_create_dto_with_minimal_fields() {
        let create_dto = UserCreateDTO {
            email: "minimal@example.com".to_string(),
            password: "password".to_string(),
            first_name: None,
            last_name: None,
            phone: None,
        };

        assert_eq!(create_dto.email, "minimal@example.com");
        assert_eq!(create_dto.password, "password");
        assert_eq!(create_dto.first_name, None);
        assert_eq!(create_dto.last_name, None);
        assert_eq!(create_dto.phone, None);
    }

    #[tokio::test]
    async fn test_user_update_dto_creation() {
        let update_dto = UserUpdateDTO {
            email: Some("updated@example.com".to_string()),
            password: Some("new_password".to_string()),
            first_name: Some("Updated".to_string()),
            last_name: Some("Name".to_string()),
            phone: Some("999-8888".to_string()),
        };

        // assert_eq!(update_dto.id, 1); // removed
        assert_eq!(update_dto.email, Some("updated@example.com".to_string()));
        assert_eq!(update_dto.password, Some("new_password".to_string()));
        assert_eq!(update_dto.first_name, Some("Updated".to_string()));
        assert_eq!(update_dto.last_name, Some("Name".to_string()));
        assert_eq!(update_dto.phone, Some("999-8888".to_string()));
    }

    #[tokio::test]
    async fn test_user_update_dto_with_partial_fields() {
        let update_dto = UserUpdateDTO {
            email: Some("partial@example.com".to_string()),
            password: None,
            first_name: None,
            last_name: Some("OnlyLast".to_string()),
            phone: None,
        };

        // assert_eq!(update_dto.id, 2); // removed
        assert_eq!(update_dto.email, Some("partial@example.com".to_string()));
        assert_eq!(update_dto.password, None);
        assert_eq!(update_dto.first_name, None);
        assert_eq!(update_dto.last_name, Some("OnlyLast".to_string()));
        assert_eq!(update_dto.phone, None);
    }

    #[tokio::test]
    async fn test_user_search_param_dto_creation() {
        let search_dto = UserSearchParamsDTO {
            email: Some("search@example.com".to_string()),
            first_name: Some("Search".to_string()),
            last_name: Some("Term".to_string()),
            page: Some(2),
            page_size: Some(20),
            order_by: Some("+created_at,-id".to_string()),
        };

        assert_eq!(search_dto.email, Some("search@example.com".to_string()));
        assert_eq!(search_dto.first_name, Some("Search".to_string()));
        assert_eq!(search_dto.last_name, Some("Term".to_string()));
        assert_eq!(search_dto.page, Some(2));
        assert_eq!(search_dto.page_size, Some(20));
        assert_eq!(search_dto.order_by, Some("+created_at,-id".to_string()));
    }

    #[tokio::test]
    async fn test_user_search_param_dto_empty() {
        let search_dto = UserSearchParamsDTO {
            email: None,
            first_name: None,
            last_name: None,
            page: None,
            page_size: None,
            order_by: None,
        };

        assert_eq!(search_dto.email, None);
        assert_eq!(search_dto.first_name, None);
        assert_eq!(search_dto.last_name, None);
        assert_eq!(search_dto.page, None);
        assert_eq!(search_dto.page_size, None);
        assert_eq!(search_dto.order_by, None);
    }

    #[tokio::test]
    async fn test_user_list_dto_creation() {
        let now = chrono::Utc::now().naive_utc();

        let user_dto1 = UserDTO {
            id: 1,
            email: "user1@example.com".to_string(),
            first_name: Some("User".to_string()),
            last_name: Some("One".to_string()),
            phone: Some("111-1111".to_string()),
            created_at: Some(now),
            updated_at: Some(now),
            created_user: None,
            updated_user: None,
        };

        let user_dto2 = UserDTO {
            id: 2,
            email: "user2@example.com".to_string(),
            first_name: Some("User".to_string()),
            last_name: Some("Two".to_string()),
            phone: Some("222-2222".to_string()),
            created_at: Some(now),
            updated_at: Some(now),
            created_user: None,
            updated_user: None,
        };

        let user_list = UserListDTO {
            items: vec![user_dto1, user_dto2],
            count: 2,
        };

        assert_eq!(user_list.items.len(), 2);
        assert_eq!(user_list.count, 2);
        assert_eq!(user_list.items[0].email, "user1@example.com");
        assert_eq!(user_list.items[1].email, "user2@example.com");
    }

    #[tokio::test]
    async fn test_user_list_dto_empty() {
        let user_list = UserListDTO {
            items: vec![],
            count: 0,
        };

        assert_eq!(user_list.items.len(), 0);
        assert_eq!(user_list.count, 0);
    }

    #[tokio::test]
    async fn test_user_dto_serialization_fields() {
        // This test ensures all required fields are present for serialization
        let now = chrono::Utc::now().naive_utc();

        let created_user = my_axum::user::dto::user_dto::UserSimpleDTO {
            id: 1,
            email: "creator@example.com".to_string(),
            first_name: Some("Creator".to_string()),
            last_name: Some("User".to_string()),
        };

        let updated_user = my_axum::user::dto::user_dto::UserSimpleDTO {
            id: 2,
            email: "updater@example.com".to_string(),
            first_name: Some("Updater".to_string()),
            last_name: Some("User".to_string()),
        };

        let user_dto = UserDTO {
            id: 1,
            email: "serialize@example.com".to_string(),
            first_name: Some("Serialize".to_string()),
            last_name: Some("Test".to_string()),
            phone: Some("123-456-7890".to_string()),
            created_at: Some(now),
            updated_at: Some(now),
            created_user: Some(created_user),
            updated_user: Some(updated_user),
        };

        // Verify all fields are accessible
        assert_eq!(user_dto.id, 1);
        assert_eq!(user_dto.email, "serialize@example.com");
        assert_eq!(user_dto.first_name.unwrap(), "Serialize");
        assert_eq!(user_dto.last_name.unwrap(), "Test");
        assert_eq!(user_dto.phone.unwrap(), "123-456-7890");
        assert!(user_dto.created_at.is_some());
        assert!(user_dto.updated_at.is_some());
        assert_eq!(user_dto.created_user.as_ref().unwrap().id, 1);
        assert_eq!(user_dto.updated_user.as_ref().unwrap().id, 2);
    }

    #[tokio::test]
    async fn test_dto_edge_cases() {
        // Test with empty strings
        let create_dto = UserCreateDTO {
            email: "".to_string(),
            password: "".to_string(),
            first_name: Some("".to_string()),
            last_name: Some("".to_string()),
            phone: Some("".to_string()),
        };

        assert_eq!(create_dto.email, "");
        assert_eq!(create_dto.password, "");
        assert_eq!(create_dto.first_name, Some("".to_string()));
        assert_eq!(create_dto.last_name, Some("".to_string()));
        assert_eq!(create_dto.phone, Some("".to_string()));

        // Test update DTO with all None values
        let update_dto = UserUpdateDTO {
            email: None,
            password: None,
            first_name: None,
            last_name: None,
            phone: None,
        };

        // Verify all fields are None
        assert!(update_dto.email.is_none());
        assert!(update_dto.password.is_none());

        // Test search with page 0 and size 0
        let search_dto = UserSearchParamsDTO {
            email: None,
            first_name: None,
            last_name: None,
            page: Some(0),
            page_size: Some(0),
            order_by: None,
        };

        assert_eq!(search_dto.page, Some(0));
        assert_eq!(search_dto.page_size, Some(0));
    }

    #[tokio::test]
    async fn test_user_dto_conversion_chain() {
        // Test the complete conversion chain from Model -> DTO -> Model fields comparison
        let now = chrono::Utc::now().naive_utc();

        let original_model = user::Model {
            id: 1,
            email: "conversion@example.com".to_string(),
            password: "secret_password".to_string(),
            first_name: Some("Conversion".to_string()),
            last_name: Some("Test".to_string()),
            phone: Some("555-CONV".to_string()),
            created_at: Some(now),
            updated_at: Some(now),
            created_user_id: Some(10),
            updated_user_id: Some(20),
        };

        // Create related user models for testing
        let created_user_model = user::Model {
            id: 10,
            email: "created_user@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Created".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
            created_at: Some(now),
            updated_at: Some(now),
            created_user_id: None,
            updated_user_id: None,
        };

        let updated_user_model = user::Model {
            id: 20,
            email: "updated_user@example.com".to_string(),
            password: "password123@".to_string(),
            first_name: Some("Updated".to_string()),
            last_name: Some("User".to_string()),
            phone: None,
            created_at: Some(now),
            updated_at: Some(now),
            created_user_id: None,
            updated_user_id: None,
        };

        // Convert to DTO
        let user_dto: UserDTO = UserWithRelations {
            model: original_model.clone(),
            created_user: Some(created_user_model),
            updated_user: Some(updated_user_model),
        }
        .into();

        // Verify the conversion preserves all data
        assert_eq!(user_dto.id, original_model.id);
        assert_eq!(user_dto.email, original_model.email);
        assert_eq!(user_dto.first_name, original_model.first_name);
        assert_eq!(user_dto.last_name, original_model.last_name);
        assert_eq!(user_dto.phone, original_model.phone);
        assert_eq!(user_dto.created_at, original_model.created_at);
        assert_eq!(user_dto.updated_at, original_model.updated_at);
        assert_eq!(user_dto.created_user.as_ref().unwrap().id, 10);
        assert_eq!(user_dto.updated_user.as_ref().unwrap().id, 20);
    }
}
