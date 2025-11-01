use crate::{
    core::context::Context,
    user::repository::refresh_token_repository::{self, RefreshTokenSearchParams},
};
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};

pub async fn clean_expired_tokens(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
    tracing::info!("Starting cleanup of expired refresh tokens");

    db.transaction::<_, (), DbErr>(|txn| {
        Box::pin(async move {
            let context = Context {
                txn,
                user: None,
                producer: None,
            };

            let expired_tokens = refresh_token_repository::search(
                &context,
                &RefreshTokenSearchParams {
                    is_expired: Some(true),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

            // Delete tokens in batches
            const BATCH_SIZE: usize = 100;
            for chunk in expired_tokens.chunks(BATCH_SIZE) {
                let tokens: Vec<String> = chunk.iter().map(|token| token.token.clone()).collect();
                refresh_token_repository::delete_by_tokens(&context, &tokens)
                    .await
                    .map_err(|e| DbErr::Custom(e.to_string()))?;
            }

            Ok(())
        })
    })
    .await?;

    Ok(())
}
