#[cfg(test)]
mod avatar_dto_tests {
    use my_axum::user::dto::avatar_dto::{AvatarUploadProgressDTO, UploadAvatarDTO};

    #[test]
    fn test_upload_avatar_dto_creation() {
        let dto = UploadAvatarDTO {
            user_id: 1,
            file_name: "avatar.jpg".to_string(),
        };

        assert_eq!(dto.user_id, 1);
        assert_eq!(dto.file_name, "avatar.jpg");
    }

    #[test]
    fn test_upload_avatar_dto_clone() {
        let dto1 = UploadAvatarDTO {
            user_id: 2,
            file_name: "profile.png".to_string(),
        };

        let dto2 = dto1.clone();
        assert_eq!(dto1.user_id, dto2.user_id);
        assert_eq!(dto1.file_name, dto2.file_name);
    }

    #[test]
    fn test_upload_avatar_dto_debug() {
        let dto = UploadAvatarDTO {
            user_id: 3,
            file_name: "image.gif".to_string(),
        };

        let debug_str = format!("{:?}", dto);
        assert!(debug_str.contains("UploadAvatarDTO"));
        assert!(debug_str.contains("user_id"));
        assert!(debug_str.contains("file_name"));
    }

    #[test]
    fn test_upload_avatar_dto_serialization() {
        let dto = UploadAvatarDTO {
            user_id: 4,
            file_name: "photo.jpg".to_string(),
        };

        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("user_id"));
        assert!(json.contains("file_name"));
        assert!(json.contains("photo.jpg"));
    }

    #[test]
    fn test_upload_avatar_dto_deserialization() {
        let json = r#"{"user_id":5,"file_name":"test.png"}"#;
        let dto: UploadAvatarDTO = serde_json::from_str(json).unwrap();

        assert_eq!(dto.user_id, 5);
        assert_eq!(dto.file_name, "test.png");
    }

    #[test]
    fn test_upload_avatar_dto_roundtrip() {
        let original = UploadAvatarDTO {
            user_id: 6,
            file_name: "avatar.webp".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: UploadAvatarDTO = serde_json::from_str(&json).unwrap();

        assert_eq!(original.user_id, deserialized.user_id);
        assert_eq!(original.file_name, deserialized.file_name);
    }

    #[test]
    fn test_upload_avatar_dto_with_special_characters() {
        let dto = UploadAvatarDTO {
            user_id: 7,
            file_name: "ảnh đại diện.jpg".to_string(),
        };

        assert_eq!(dto.file_name, "ảnh đại diện.jpg");
    }

    #[test]
    fn test_upload_avatar_dto_with_long_filename() {
        let long_name = "a".repeat(255);
        let dto = UploadAvatarDTO {
            user_id: 8,
            file_name: long_name.clone(),
        };

        assert_eq!(dto.file_name.len(), 255);
    }

    #[test]
    fn test_avatar_upload_progress_dto_new() {
        let dto = AvatarUploadProgressDTO::new("task-10".to_string(), 10, 50, "uploading");

        assert_eq!(dto.task_id, "task-10");
        assert_eq!(dto.user_id, 10);
        assert_eq!(dto.progress, 50);
        assert_eq!(dto.status, "uploading");
        assert_eq!(dto.message, None);
    }

    #[test]
    fn test_avatar_upload_progress_dto_with_message() {
        let dto = AvatarUploadProgressDTO::new("task-11".to_string(), 11, 75, "processing")
            .with_message("Converting image format");

        assert_eq!(dto.task_id, "task-11");
        assert_eq!(dto.user_id, 11);
        assert_eq!(dto.progress, 75);
        assert_eq!(dto.status, "processing");
        assert_eq!(dto.message, Some("Converting image format".to_string()));
    }

    #[test]
    fn test_avatar_upload_progress_dto_clone() {
        let dto1 = AvatarUploadProgressDTO::new("task-12".to_string(), 12, 100, "completed")
            .with_message("Upload successful");

        let dto2 = dto1.clone();
        assert_eq!(dto1.task_id, dto2.task_id);
        assert_eq!(dto1.user_id, dto2.user_id);
        assert_eq!(dto1.progress, dto2.progress);
        assert_eq!(dto1.status, dto2.status);
        assert_eq!(dto1.message, dto2.message);
    }

    #[test]
    fn test_avatar_upload_progress_dto_debug() {
        let dto = AvatarUploadProgressDTO::new("task-13".to_string(), 13, 25, "pending");

        let debug_str = format!("{:?}", dto);
        assert!(debug_str.contains("AvatarUploadProgressDTO"));
        assert!(debug_str.contains("task_id"));
        assert!(debug_str.contains("user_id"));
        assert!(debug_str.contains("progress"));
        assert!(debug_str.contains("status"));
    }

    #[test]
    fn test_avatar_upload_progress_dto_serialization() {
        let dto = AvatarUploadProgressDTO::new("task-14".to_string(), 14, 80, "uploading")
            .with_message("80% complete");

        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("task_id"));
        assert!(json.contains("user_id"));
        assert!(json.contains("progress"));
        assert!(json.contains("status"));
        assert!(json.contains("message"));
    }

    #[test]
    fn test_avatar_upload_progress_dto_deserialization() {
        let json = r#"{"task_id":"task-15","user_id":15,"progress":90,"status":"finalizing","message":"Almost done"}"#;
        let dto: AvatarUploadProgressDTO = serde_json::from_str(json).unwrap();

        assert_eq!(dto.task_id, "task-15");
        assert_eq!(dto.user_id, 15);
        assert_eq!(dto.progress, 90);
        assert_eq!(dto.status, "finalizing");
        assert_eq!(dto.message, Some("Almost done".to_string()));
    }

    #[test]
    fn test_avatar_upload_progress_dto_without_message() {
        let json =
            r#"{"task_id":"task-16","user_id":16,"progress":0,"status":"starting","message":null}"#;
        let dto: AvatarUploadProgressDTO = serde_json::from_str(json).unwrap();

        assert_eq!(dto.task_id, "task-16");
        assert_eq!(dto.user_id, 16);
        assert_eq!(dto.progress, 0);
        assert_eq!(dto.status, "starting");
        assert_eq!(dto.message, None);
    }

    #[test]
    fn test_avatar_upload_progress_dto_all_progress_values() {
        for progress in 0..=100u8 {
            let dto = AvatarUploadProgressDTO::new("task-17".to_string(), 17, progress, "testing");
            assert_eq!(dto.progress, progress);
        }
    }

    #[test]
    fn test_avatar_upload_progress_dto_various_statuses() {
        let statuses = ["pending", "uploading", "processing", "completed", "failed"];

        for (i, status) in statuses.iter().enumerate() {
            let dto =
                AvatarUploadProgressDTO::new(format!("task-{}", 18 + i), 18 + i as i32, 50, status);
            assert_eq!(dto.status, *status);
        }
    }

    #[test]
    fn test_avatar_upload_progress_dto_chaining_methods() {
        let dto = AvatarUploadProgressDTO::new("task-25".to_string(), 25, 100, "completed")
            .with_message("Upload finished successfully");

        assert_eq!(dto.task_id, "task-25");
        assert_eq!(dto.user_id, 25);
        assert_eq!(dto.progress, 100);
        assert_eq!(dto.status, "completed");
        assert!(dto.message.is_some());
    }

    #[test]
    fn test_avatar_upload_progress_dto_empty_message() {
        let dto = AvatarUploadProgressDTO::new("task-26".to_string(), 26, 50, "processing")
            .with_message("");

        assert_eq!(dto.message, Some("".to_string()));
    }

    #[test]
    fn test_avatar_upload_progress_dto_long_message() {
        let long_msg = "A".repeat(500);
        let dto = AvatarUploadProgressDTO::new("task-27".to_string(), 27, 60, "uploading")
            .with_message(&long_msg);

        assert_eq!(dto.message.as_ref().unwrap().len(), 500);
    }

    #[test]
    fn test_avatar_upload_progress_dto_unicode_message() {
        let dto = AvatarUploadProgressDTO::new("task-28".to_string(), 28, 70, "processing")
            .with_message("Đang xử lý hình ảnh 🖼️");

        assert!(dto.message.as_ref().unwrap().contains("Đang xử lý"));
    }

    #[test]
    fn test_avatar_upload_progress_dto_multiple_with_message_calls() {
        let dto = AvatarUploadProgressDTO::new("task-29".to_string(), 29, 85, "processing")
            .with_message("First message")
            .with_message("Second message");

        // The last message should be retained
        assert_eq!(dto.message, Some("Second message".to_string()));
    }
}
