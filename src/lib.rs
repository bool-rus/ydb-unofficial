#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
//pub mod generated;
pub mod client;
#[cfg(feature = "pool")]
#[cfg_attr(docsrs, doc(cfg(feature = "pool")))]
pub mod pool;
pub mod auth;
pub mod error;

mod payload;
mod reimport;

pub use payload::YdbResponseWithResult;
pub use client::YdbConnection;
pub use client::YdbTransaction;
pub use reimport::*;
