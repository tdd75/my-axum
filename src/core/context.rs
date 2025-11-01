use sea_orm::DatabaseTransaction;
use std::sync::Arc;

use crate::core::db::entity::user;
use crate::pkg::messaging::MessageProducer;

pub struct ContextBuilder {
    txn_inner: Arc<DatabaseTransaction>,
    user: Option<user::Model>,
    producer: Option<Arc<Box<dyn MessageProducer>>>,
    locale: Option<String>,
}

impl ContextBuilder {
    pub fn user(mut self, user: user::Model) -> Self {
        self.user = Some(user);
        self
    }

    pub fn producer(mut self, producer: Arc<Box<dyn MessageProducer>>) -> Self {
        self.producer = Some(producer);
        self
    }

    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }

    pub fn build(self) -> Context {
        Context {
            txn_inner: self.txn_inner,
            user: self.user,
            producer: self.producer,
            locale: self.locale.unwrap_or_else(|| "en".to_string()),
        }
    }
}

#[derive(Clone)]
pub struct Context {
    txn_inner: Arc<DatabaseTransaction>,
    pub user: Option<user::Model>,
    pub producer: Option<Arc<Box<dyn MessageProducer>>>,
    pub locale: String,
}

impl Context {
    pub fn builder(txn: Arc<DatabaseTransaction>) -> ContextBuilder {
        ContextBuilder {
            txn_inner: txn,
            user: None,
            producer: None,
            locale: None,
        }
    }

    pub fn txn(&self) -> &DatabaseTransaction {
        &self.txn_inner
    }

    /// Commit the underlying transaction (or savepoint).
    /// Consumes `self` so the Arc can be unwrapped.
    pub async fn commit(self) -> Result<(), sea_orm::DbErr> {
        match Arc::try_unwrap(self.txn_inner) {
            Ok(txn) => txn.commit().await,
            Err(_) => Err(sea_orm::DbErr::Custom(
                "Failed to unwrap transaction Arc for commit".to_string(),
            )),
        }
    }
}
