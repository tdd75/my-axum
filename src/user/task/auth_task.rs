use crate::{
    core::context::Context,
    user::repository::refresh_token_repository::{self, RefreshTokenSearchParams},
};
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub async fn clean_expired_tokens(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
    tracing::info!("Starting cleanup of expired refresh tokens");

    let txn = db.begin().await?;
    let txn = Arc::new(txn);
    let context = Context::builder(txn.clone()).build();

    let expired_tokens = refresh_token_repository::search(
        &context,
        &RefreshTokenSearchParams {
            is_expired: Some(true),
            ..Default::default()
        },
    )
    .await?;

    // Delete tokens in batches
    const BATCH_SIZE: usize = 100;
    for chunk in expired_tokens.chunks(BATCH_SIZE) {
        let tokens: Vec<String> = chunk.iter().map(|token| token.token.clone()).collect();
        refresh_token_repository::delete_by_tokens(&context, &tokens).await?;
    }

    drop(context);
    Arc::try_unwrap(txn)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap transaction for commit"))?
        .commit()
        .await?;

    Ok(())
}
