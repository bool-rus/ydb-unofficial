//! Migration implementation for Ydb
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
//!     let path = std::path::Path::new("test/migrations");
//!     let migrator = Migrator::new(path).await?;
//!     migrator.run_direct(&mut conn).await?;
//! #   Ok(())
//! # }
//! ```
use std::time::Duration;

use futures::future::{BoxFuture, ok};
use sqlx_core::encode::Encode;
use sqlx_core::migrate::*;

use super::prelude::{query, query_as, YdbError, Ydb, YdbConnection};

impl From<YdbError> for MigrateError {
    fn from(value: YdbError) -> Self {
        let sqlx_error = sqlx_core::Error::from(value);
        sqlx_error.into()
    }
}


impl Migrate for YdbConnection {
    fn ensure_migrations_table(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> { Box::pin(async move {
        query(r#"
            create table _sqlx_migrations (
                version Int64,
                description Utf8,
                checksum String,
                installed_on Timestamp,
                success Bool,
                PRIMARY KEY (version)
            );
        "#).execute(self.scheme_executor()?).await?;
        Ok(())
    })}

    fn dirty_version(&mut self) -> BoxFuture<'_, Result<Option<i64>, MigrateError>> { Box::pin(async move {
        let id = query_as::<_, (i64,)>(
            "select version from (select version, installed_on from _sqlx_migrations where success = false order by installed_on desc limit 1);"
        )
        .fetch_optional(self.executor()?).await?;
        Ok(id.map(|(x,)|x))
    })}

    fn list_applied_migrations(&mut self) -> BoxFuture<'_, Result<Vec<AppliedMigration>, MigrateError>> { Box::pin(async move {
        let migrations = query_as::<_,(i64,Vec<u8>)>("select version, checksum from _sqlx_migrations;").fetch_all(self.executor()?).await?;
        Ok(migrations.into_iter().map(|(version, checksum)|AppliedMigration{ version, checksum: std::borrow::Cow::Owned(checksum) }).collect())
    })}

    fn lock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(ok(()))
    }

    fn unlock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(ok(()))
    }

    fn apply<'e: 'm, 'm>(&'e mut self, migration: &'m Migration) -> BoxFuture<'m, Result<Duration, MigrateError>> { Box::pin(async move {
        let upsert_yql = r#"
            declare $version as Int64;
            declare $description as Utf8;
            declare $checksum as String;
            declare $success as Bool;
            $installed_on = CurrentUtcTimestamp(0);
            upsert into _sqlx_migrations    ( version,  description,  checksum,  installed_on,  success) 
                                    values  ($version, $description, $checksum, $installed_on, $success);
        "#;
        query(upsert_yql).bind(migration).execute(self.executor()?).await?;
        let now = std::time::Instant::now();
        query(&migration.sql).execute(self.scheme_executor()?).await?;
        let elapsed = now.elapsed();
        query(upsert_yql).bind(migration)
            .bind(("$success", true)).execute(self.executor()?).await?;
        Ok(elapsed)
    })}

    fn revert<'e: 'm, 'm>(&'e mut self, _migration: &'m Migration) -> BoxFuture<'m, Result<Duration, MigrateError>> {
        unimplemented!()
    }
}

impl<'q> Encode<'q, Ydb> for Migration {
    fn encode_by_ref(&self, buf: &mut <Ydb as sqlx_core::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx_core::encode::IsNull {
        let Migration { version, description, checksum, .. } = self;
        let _ = ("$version", *version).encode(buf);
        let _ = ("$description", description.to_string()).encode(buf);
        let _ = ("$checksum", checksum.to_vec()).encode(buf);
        let _ = ("$success", false).encode(buf);
        sqlx_core::encode::IsNull::No
    }
}

impl sqlx_core::types::Type<Ydb> for Migration {
    fn type_info() -> <Ydb as sqlx_core::database::Database>::TypeInfo {
        unimplemented!()
    }
}
