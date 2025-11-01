#[macro_use]
extern crate rust_i18n;
i18n!("src/core/translation/locales");

pub mod assistant;
pub mod common;
pub mod config;
pub mod core;
pub mod user;
pub use pkg;
