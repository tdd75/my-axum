mod context_creation_tests {
    use crate::setup::app::TestApp;
    use my_axum::core::context::Context;
    use sea_orm::TransactionTrait;

    #[tokio::test]
    async fn test_context_creation_without_user() {
        let app = TestApp::spawn_app().await;
        let db = &app.db;

        let result = db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    assert!(context.user.is_none());
                    assert!(context.producer.is_none());
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_producer_none() {
        let app = TestApp::spawn_app().await;
        let db = &app.db;

        let result = db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    assert!(context.producer.is_none());
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_multiple_instances() {
        let app = TestApp::spawn_app().await;
        let db = &app.db;

        let result = db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context1 = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let context2 = Context {
                        txn,
                        user: None,
                        producer: None,
                    };

                    assert!(context1.user.is_none());
                    assert!(context2.user.is_none());
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
    }
}

mod context_field_tests {
    use crate::setup::app::TestApp;
    use my_axum::core::context::Context;
    use sea_orm::TransactionTrait;

    #[tokio::test]
    async fn test_context_struct_has_txn_field() {
        let app = TestApp::spawn_app().await;
        let db = &app.db;

        let result = db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    // Txn field exists and can be accessed
                    let _txn_ref = context.txn;
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_struct_has_user_field() {
        let app = TestApp::spawn_app().await;
        let db = &app.db;

        let result = db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    // User field exists and is accessible
                    assert!(context.user.is_none());
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_struct_has_producer_field() {
        let app = TestApp::spawn_app().await;
        let db = &app.db;

        let result = db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    // Producer field exists and is accessible
                    assert!(context.producer.is_none());
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_user_option_type() {
        let app = TestApp::spawn_app().await;
        let db = &app.db;

        let result = db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    // User is Option type
                    if context.user.is_some() {
                        panic!("User should be None");
                    }
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
    }
}

mod context_behavior_tests {
    use crate::setup::app::TestApp;
    use my_axum::core::context::Context;
    use sea_orm::TransactionTrait;

    #[tokio::test]
    async fn test_context_clone() {
        let app = TestApp::spawn_app().await;
        let db = &app.db;

        let result = db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context1 = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    let context2 = context1.clone();
                    assert!(context2.user.is_none());
                    assert!(context2.producer.is_none());
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_can_be_cloned() {
        let app = TestApp::spawn_app().await;
        let db = &app.db;

        let result = db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let context1 = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    // Context can be cloned
                    let _context2 = context1.clone();
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_context_in_transaction() {
        let app = TestApp::spawn_app().await;
        let db = &app.db;

        // Test that context can be used within a transaction
        let result = db
            .transaction::<_, (), sea_orm::DbErr>(|txn| {
                Box::pin(async move {
                    let _context = Context {
                        txn,
                        user: None,
                        producer: None,
                    };
                    Ok(())
                })
            })
            .await;

        assert!(result.is_ok());
    }
}
