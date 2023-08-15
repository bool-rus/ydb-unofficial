mod dumb;
mod entities;
mod database;
mod error;
mod connection;
mod executor;
mod types;
//TODO: спрятать под фичу
mod minikql;

mod statement;

type YdbConnection = crate::YdbConnection<crate::auth::UpdatableToken>;
//pub use query::*;
pub use database::*;
pub use entities::*;
pub use statement::*;
pub use super::error::YdbError;