use sea_orm::DatabaseTransaction;
use std::sync::Arc;

use crate::core::db::entity::user;
use crate::pkg::messaging::MessageProducer;

#[derive(Clone)]
pub struct Context<'a> {
    pub txn: &'a DatabaseTransaction,
    pub user: Option<user::Model>,
    pub producer: Option<Arc<Box<dyn MessageProducer>>>,
}
