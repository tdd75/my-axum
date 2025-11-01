use sea_orm::TransactionTrait;
use std::backtrace::Backtrace;
use std::sync::Arc;

use crate::{
    config::app::AppState,
    core::{context::Context, db::entity::user},
};

/// Helper function to execute a use case within a transaction
/// Automatically handles commit/rollback based on the result
pub async fn new_transaction<T, F, E>(
    app_state: &AppState,
    current_user: Option<user::Model>,
    locale: Option<String>,
    use_case_fn: F,
) -> Result<T, E>
where
    T: Send,
    E: From<sea_orm::DbErr> + std::fmt::Display + std::fmt::Debug + Send,
    F: FnOnce(
            &Context,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send + '_>>
        + Send,
{
    let txn = app_state.db.begin().await.map_err(|e| {
        let backtrace = Backtrace::capture();
        tracing::error!(
            error = %e,
            backtrace = %backtrace,
            "Failed to begin transaction"
        );
        E::from(e)
    })?;

    let txn = Arc::new(txn);
    let producer = app_state.producer.clone();
    let mut context_builder = Context::builder(txn.clone());
    if let Some(locale) = locale {
        context_builder = context_builder.locale(locale);
    }
    if let Some(user) = current_user {
        context_builder = context_builder.user(user);
    }
    if let Some(producer) = producer {
        context_builder = context_builder.producer(producer);
    }
    let context = context_builder.build();

    let result = use_case_fn(&context).await;
    drop(context);

    match result {
        Ok(value) => {
            match Arc::try_unwrap(txn) {
                Ok(txn) => {
                    txn.commit().await.map_err(|e| {
                        let backtrace = Backtrace::capture();
                        tracing::error!(
                            error = %e,
                            backtrace = %backtrace,
                            "Transaction commit error"
                        );
                        E::from(e)
                    })?;
                }
                Err(_) => {
                    tracing::error!("Failed to unwrap transaction Arc for commit");
                    return Err(E::from(sea_orm::DbErr::Custom(
                        "Failed to commit transaction".to_string(),
                    )));
                }
            }
            Ok(value)
        }
        Err(e) => {
            let backtrace = Backtrace::capture();
            tracing::error!(
                error = %e,
                backtrace = %backtrace,
                "Transaction error occurred"
            );
            Err(e)
        }
    }
}
