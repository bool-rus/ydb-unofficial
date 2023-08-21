mod entities;
mod database;
mod error;

/// SQLX Connection implementation for Ydb
/// 
/// # Examples
/// 
/// ``` rust
/// # #[tokio::main]
/// # async fn main() {
/// use ydb_unofficial::sqlx::*;
/// use sqlx::prelude::*;
/// use std::str::FromStr;
/// let options = YdbConnectOptions::from_str("grpcs://ydb.serverless.yandexcloud.net:2135/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f?sa-key=test-env/authorized_key.json").unwrap();
/// let mut conn = options.connect().await.unwrap();
/// let row: (i32,) = sqlx::query_as("declare $one as Int32; select $one+$one as sum;").bind(("$one",1)).fetch_one(
///     conn.executor().unwrap()
/// ).await.unwrap();
/// assert_eq!(row.0, 2);
/// 
/// # }
/// ```
mod connection;
mod executor;
mod types;
//TODO: спрятать под фичу
mod minikql;

mod statement;

pub use database::*;
pub use entities::*;
pub use statement::*;
pub use connection::*;
pub use super::error::YdbError;