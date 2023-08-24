//! SQLX traits implementation for Ydb
//! 
//! # Examples
//! 
//! ``` rust
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     use ydb_unofficial::sqlx::prelude::*;
//!     let token = std::env::var("DB_TOKEN").unwrap();
//!     let conn_str = format!("ydbs://ydb.serverless.yandexcloud.net:2135/ru-central1/b1gtv82sacrcnutlfktm/etn8sgrgdbp7jqv64k9f?token={token}");
//!     let options = YdbConnectOptions::from_str(&conn_str)?;
//!     let mut conn = options.connect().await?;
//!     query("create table my_test_table (id int32, text utf8, primary key(id));")
//!         .execute(conn.scheme_executor()?).await?;
//!     let result = query("declare $id as int32; declare $text as utf8; upsert into my_test_table(id, text) values($id, $text);")
//!         .bind(("$id",1))
//!         .bind(("$text", "one".to_owned()))
//!         .execute(conn.executor()?).await?;
//!     let row: (i32, String) = query_as("select * from my_test_table;")
//!         .fetch_one(conn.executor()?).await?;
//!     assert_eq!(row.0, 1);
//!     assert_eq!(row.1, "one");
//!     query("drop table my_test_table;")
//!         .execute(conn.scheme_executor()?).await?;
//! #   Ok(())
//! # }
//! ```

pub mod entities;
pub mod database;
pub mod error;

pub mod connection;
pub mod executor;
pub mod types;
pub mod statement;
//TODO: спрятать под фичу
mod minikql;

use super::error::YdbError;

pub mod prelude;

#[cfg(feature = "migrate")]
#[cfg_attr(docsrs, doc(cfg(feature = "migrate")))]
mod migration;