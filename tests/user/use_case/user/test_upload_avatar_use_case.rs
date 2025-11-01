#[cfg(test)]
mod upload_avatar_use_case_tests {
    use crate::setup::app::TestApp;
    use async_trait::async_trait;
    use my_axum::{
        core::{
            context::Context,
            db::entity::{sea_orm_active_enums::UserRole, user},
            task::{TaskEvent, TaskType},
        },
        pkg::messaging::MessageProducer,
        user::{
            dto::{
                avatar_dto::{UploadAvatarDTO, UploadAvatarResponseDTO},
                user_dto::{UserCreateDTO, UserDTO},
            },
            use_case::{user::create_user_use_case, user::upload_avatar_use_case},
        },
    };
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockProducer {
        pub published_messages: Arc<Mutex<Vec<String>>>,
        pub should_fail: bool,
    }

    impl MockProducer {
        fn new() -> Self {
            Self {
                published_messages: Arc::new(Mutex::new(Vec::new())),
                should_fail: false,
            }
        }

        fn new_failing() -> Self {
            Self {
                published_messages: Arc::new(Mutex::new(Vec::new())),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl MessageProducer for MockProducer {
        async fn publish_event_json(
            &self,
            event_json: &str,
            _destination: Option<&str>,
        ) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("Mock producer failure"));
            }

            self.published_messages
                .lock()
                .unwrap()
                .push(event_json.to_string());
            Ok(())
        }
    }

    fn build_user_model(user_dto: &UserDTO, role: UserRole) -> user::Model {
        user::Model {
            id: user_dto.id,
            email: user_dto.email.clone(),
            password: "hashed_password".to_string(),
            role,
            first_name: user_dto.first_name.clone(),
            last_name: user_dto.last_name.clone(),
            phone: user_dto.phone.clone(),
            created_at: user_dto.created_at,
            updated_at: user_dto.updated_at,
            created_user_id: None,
            updated_user_id: None,
        }
    }

    async fn create_user(context: &Context, email: &str) -> UserDTO {
        create_user_use_case::execute(
            context,
            UserCreateDTO {
                email: email.to_string(),
                password: "password123@".to_string(),
                first_name: Some("Avatar".to_string()),
                last_name: Some("User".to_string()),
                phone: None,
            },
        )
        .await
        .unwrap()
        .data
    }

    async fn authenticate_context(
        context: &mut Context,
        email: &str,
        role: UserRole,
    ) -> user::Model {
        let user = create_user(context, email).await;
        let user_model = build_user_model(&user, role);
        context.user = Some(user_model.clone());
        user_model
    }

    fn upload_request(file_name: &str) -> UploadAvatarDTO {
        UploadAvatarDTO {
            file_name: file_name.to_string(),
        }
    }

    fn assert_accepted_response(response: &UploadAvatarResponseDTO) {
        assert!(!response.task_id.is_empty());
        assert!(uuid::Uuid::parse_str(&response.task_id).is_ok());
        assert!(response.message.contains("Avatar upload initiated"));
        assert!(response.message.contains(&response.task_id));
    }

    fn parse_published_task(message: &str) -> TaskEvent {
        serde_json::from_str(message).unwrap()
    }

    #[tokio::test]
    async fn test_upload_avatar_requires_authenticated_user() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let context = Context::builder(Arc::new(txn)).build();

        let result =
            upload_avatar_use_case::execute(&context, upload_request("avatar.jpg"), "en").await;

        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 401);
    }

    #[tokio::test]
    async fn test_upload_avatar_uses_authenticated_user_without_producer() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let mut context = Context::builder(Arc::new(txn)).build();

        authenticate_context(&mut context, "self@example.com", UserRole::User).await;

        let result =
            upload_avatar_use_case::execute(&context, upload_request("avatar.jpg"), "en").await;

        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 500);
        assert!(error.message.contains("Producer not available"));
    }

    #[tokio::test]
    async fn test_upload_avatar_fails_when_authenticated_user_is_missing_in_database() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let mut context = Context::builder(Arc::new(txn)).build();

        context.user = Some(user::Model {
            id: 999999,
            email: "missing@example.com".to_string(),
            password: "hashed_password".to_string(),
            role: UserRole::User,
            first_name: None,
            last_name: None,
            phone: None,
            created_at: None,
            updated_at: None,
            created_user_id: None,
            updated_user_id: None,
        });

        let result =
            upload_avatar_use_case::execute(&context, upload_request("avatar.jpg"), "en").await;

        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 404);
    }

    #[tokio::test]
    async fn test_upload_avatar_success_uses_authenticated_user() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let mut context = Context::builder(Arc::new(txn)).build();

        let current_user =
            authenticate_context(&mut context, "current@example.com", UserRole::User).await;
        let _other_user = create_user(&context, "other@example.com").await;

        let mock_producer = MockProducer::new();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer.clone()));
        context.producer = Some(producer);

        let result = upload_avatar_use_case::execute(&context, upload_request("avatar.jpg"), "vi")
            .await
            .unwrap();

        assert_eq!(result.status.as_u16(), 202);
        assert_accepted_response(&result.data);

        let messages = mock_producer.published_messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        match parse_published_task(&messages[0]).task {
            TaskType::ProcessAvatarUpload {
                user_id,
                file_name,
                locale,
                ..
            } => {
                assert_eq!(user_id, current_user.id);
                assert_eq!(file_name, "avatar.jpg");
                assert_eq!(locale, "vi");
            }
            other => panic!("unexpected task published: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_upload_avatar_admin_still_targets_authenticated_user() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let mut context = Context::builder(Arc::new(txn)).build();

        let admin_user =
            authenticate_context(&mut context, "admin@example.com", UserRole::Admin).await;
        let _other_user = create_user(&context, "target@example.com").await;

        let mock_producer = MockProducer::new();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer.clone()));
        context.producer = Some(producer);

        let result =
            upload_avatar_use_case::execute(&context, upload_request("admin-avatar.jpg"), "vi")
                .await
                .unwrap();

        assert_eq!(result.status.as_u16(), 202);

        let messages = mock_producer.published_messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        match parse_published_task(&messages[0]).task {
            TaskType::ProcessAvatarUpload {
                user_id,
                file_name,
                locale,
                ..
            } => {
                assert_eq!(user_id, admin_user.id);
                assert_eq!(file_name, "admin-avatar.jpg");
                assert_eq!(locale, "vi");
            }
            other => panic!("unexpected task published: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_upload_avatar_surfaces_producer_publish_errors() {
        let test_app = TestApp::spawn_app().await;
        let txn = test_app.begin_transaction().await;
        let mut context = Context::builder(Arc::new(txn)).build();

        authenticate_context(&mut context, "producer-error@example.com", UserRole::User).await;

        let mock_producer = MockProducer::new_failing();
        let producer: Arc<Box<dyn MessageProducer>> = Arc::new(Box::new(mock_producer));
        context.producer = Some(producer);

        let result =
            upload_avatar_use_case::execute(&context, upload_request("avatar.jpg"), "en").await;

        let error = result.unwrap_err();
        assert_eq!(error.status.as_u16(), 500);
        assert!(error.message.contains("Failed to publish upload task"));
    }
}
