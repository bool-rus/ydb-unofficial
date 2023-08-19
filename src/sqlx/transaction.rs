use sqlx_core::transaction::TransactionManager;

use super::{Ydb, YdbConnection};

struct YdbTransactionManager;

impl TransactionManager for YdbTransactionManager {
    type Database = Ydb;

    fn begin(
        conn: &mut YdbConnection,
    ) -> futures::future::BoxFuture<'_, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn commit(
        conn: &mut YdbConnection,
    ) -> futures::future::BoxFuture<'_, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn rollback(
        conn: &mut YdbConnection,
    ) -> futures::future::BoxFuture<'_, Result<(), sqlx_core::Error>> {
        todo!()
    }

    fn start_rollback(conn: &mut YdbConnection) {
        todo!()
    }
}