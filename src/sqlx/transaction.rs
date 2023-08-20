use sqlx_core::transaction::TransactionManager;

use super::{Ydb, YdbConnection};
use futures::{future::BoxFuture, TryFutureExt};

pub struct YdbTransactionManager;

impl TransactionManager for YdbTransactionManager {
    type Database = Ydb;

    fn begin(conn: &mut YdbConnection) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {
        Box::pin(conn.begin_tx().map_err(Into::into))
    }

    fn commit(conn: &mut YdbConnection) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {
        Box::pin(conn.commit_tx().map_err(Into::into))
    }

    fn rollback(conn: &mut YdbConnection) -> BoxFuture<'_, Result<(), sqlx_core::Error>> {
        Box::pin(conn.rollback_tx().map_err(Into::into))
    }

    fn start_rollback(_conn: &mut YdbConnection) {
        log::error!("start_rollback method is unimplemented");
    }
}