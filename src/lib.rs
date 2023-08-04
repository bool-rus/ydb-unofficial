#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
mod reimport;
pub mod auth;
pub mod error;
mod payload;
pub mod client;


pub use payload::YdbResponseWithResult;
pub use client::YdbConnection;
pub use client::YdbTransaction;
pub use reimport::*;

#[cfg(feature = "pool")]
#[cfg_attr(docsrs, doc(cfg(feature = "pool")))]
pub mod pool;

#[cfg(feature = "sqlx")]
pub mod sqlx;

#[cfg(test)]
mod test;