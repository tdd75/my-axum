use std::backtrace::Backtrace;
use std::sync::Arc;

use axum::extract::State;
use axum::{extract::Request, middleware::Next, response::Response};
use sea_orm::TransactionTrait;

use crate::config::app::AppState;
use crate::core::context::Context;
use crate::core::dto::error_dto::ErrorDTO;
use crate::core::layer::lang_layer::RequestLocale;
use crate::user::entity::user;

pub async fn transaction_middleware(
    State(app_state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ErrorDTO> {
    let txn = app_state.db.begin().await.map_err(|e| {
        let backtrace = Backtrace::capture();
        tracing::error!(error = %e, backtrace = %backtrace, "Failed to begin transaction");
        ErrorDTO::from(e)
    })?;

    let txn = Arc::new(txn);
    let current_user = req.extensions().get::<user::Model>().cloned();
    let locale = req
        .extensions()
        .get::<RequestLocale>()
        .map(|l| l.as_str().to_string());
    let mut context_builder = Context::builder(txn.clone());
    if let Some(locale) = locale {
        context_builder = context_builder.locale(locale);
    }
    if let Some(current_user) = current_user {
        context_builder = context_builder.user(current_user);
    }
    if let Some(producer) = app_state.producer.clone() {
        context_builder = context_builder.producer(producer);
    }
    let context = context_builder.build();

    req.extensions_mut().insert(context);

    let response = next.run(req).await;

    match Arc::try_unwrap(txn) {
        Ok(txn) => {
            if response.status().is_success() || response.status().is_redirection() {
                txn.commit().await.map_err(|e| {
                    let backtrace = Backtrace::capture();
                    tracing::error!(error = %e, backtrace = %backtrace, "Transaction commit error");
                    ErrorDTO::from(e)
                })?;
            } else {
                txn.rollback().await.map_err(|e| {
                    let backtrace = Backtrace::capture();
                    tracing::error!(error = %e, backtrace = %backtrace, "Transaction rollback error");
                    ErrorDTO::from(e)
                })?;
            }
        }
        Err(_) => {
            tracing::error!("Transaction Arc still has multiple references, will auto-rollback");
        }
    }

    Ok(response)
}
