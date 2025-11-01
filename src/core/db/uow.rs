use sea_orm::{TransactionError, TransactionTrait};
use std::backtrace::Backtrace;

use crate::{
    config::app::AppState,
    core::{context::Context, db::entity::user},
};

/// Helper function to execute a use case within a transaction
/// Automatically handles commit/rollback based on the result
pub async fn new_transaction<T, F, E>(
    app_state: &AppState,
    current_user: Option<user::Model>,
    use_case_fn: F,
) -> Result<T, E>
where
    T: Send,
    E: From<sea_orm::DbErr> + std::fmt::Display + std::fmt::Debug + Send,
    F: for<'a> FnOnce(
            &'a Context<'a>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'a>,
        > + Send
        + 'static,
{
    let producer = app_state.producer.clone();
    app_state
        .db
        .transaction::<_, T, E>(|txn| {
            Box::pin(async move {
                let context = Context {
                    txn,
                    user: current_user,
                    producer,
                };
                use_case_fn(&context).await
            })
        })
        .await
        .map_err(|e| {
            let backtrace = Backtrace::capture();
            tracing::error!(
                error = %e,
                backtrace = %backtrace,
                "Transaction error occurred"
            );

            match e {
                TransactionError::Transaction(e) => e,
                TransactionError::Connection(db_err) => db_err.into(),
            }
        })
}
