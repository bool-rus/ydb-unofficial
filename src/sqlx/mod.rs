//! SQLX traits implementation for Ydb
//! 
//! # Examples
//! 
//! ``` rust
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     use ydb_unofficial::sqlx::prelude::*;
//!     let token = std::env::var("DB_TOKEN").unwrap();
//!     let db_url = std::env::var("YDB_URL").unwrap();
//!     let db_name = std::env::var("DB_NAME").unwrap();
//!     let conn_str = format!("{db_url}{db_name}?token={token}");
//!     let options = YdbConnectOptions::from_str(&conn_str)?;
//!     let mut conn = options.connect().await?;
//!     query("create table my_test_table (id int32, text utf8, obj Json, primary key(id));")
//!         .execute(conn.scheme_executor()?).await?;
//!     let result = query("declare $id as int32; declare $text as utf8; declare $obj as Json; upsert into my_test_table(id, text, obj) values($id, $text, $obj);")
//!         .bind(("$id",1))
//!         .bind(("$text", "one".to_owned()))
//!         .bind(("$obj", Json::from("{\"x\":\"y\"}".to_owned())))
//!         .execute(conn.executor()?).await?;
//!     let row: (i32, String, Json) = query_as("select id, text, obj from my_test_table;")
//!         .fetch_one(conn.executor()?).await?;
//!     assert_eq!(row.0, 1);
//!     assert_eq!(row.1, "one");
//!     println!("row: {}", row.2.text());
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
