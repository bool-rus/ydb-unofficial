use futures::future::FutureExt;
use sqlx_core::describe::Describe;
use sqlx_core::executor::{Executor, Execute};
use sqlx_core::Either;
use tonic::codegen::futures_core::{future::BoxFuture, stream::BoxStream};
use ydb_grpc_bindings::generated::ydb::table::transaction_control::TxSelector;
use ydb_grpc_bindings::generated::ydb::table::transaction_settings::TxMode;
use ydb_grpc_bindings::generated::ydb::table::{ExecuteDataQueryRequest, TransactionControl, TransactionSettings, OnlineModeSettings};

use crate::YdbResponseWithResult;
use crate::error::YdbError;
use crate::{client::TableClientWithSession, auth::UpdatableToken};

use super::convert::Streamed;
use super::{Ydb, YdbResultSet, YdbRow, YdbTypeInfo, YdbStatement, YdbQueryResult};
type YdbExecutor<'c> = TableClientWithSession<'c, UpdatableToken>;

impl<'c> Executor<'c> for YdbExecutor<'c> {
    type Database = Ydb;

    fn execute<'e, 'q: 'e, E: 'q>(mut self, query: E,) -> BoxFuture<'e, Result<YdbQueryResult, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> {
        let yql = query.sql().to_owned();
        let query = Some(crate::generated::ydb::table::query::Query::YqlText(yql));
        let query = Some(crate::generated::ydb::table::Query{query});
        let tx_control = Some(TransactionControl { 
            commit_tx: true, 
            tx_selector: Some(TxSelector::BeginTx(TransactionSettings { 
                tx_mode: Some(TxMode::SerializableReadWrite(Default::default())) 
            }))
        });
        Box::pin(async move {
            let response = self.execute_data_query(ExecuteDataQueryRequest{ query, tx_control, ..Default::default()}).await?;
            let result = response.into_inner().result().map_err(YdbError::from)?;
            Ok(result.into())
        })
    }

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<'e, Result<Either<YdbQueryResult, YdbRow>,sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Ydb> { 
        Box::pin(Streamed::new(self.execute(query).map(|r|r.map(|qr|{
            let cap = qr.result_sets.iter().fold(0, |sum, rs|sum + rs.rows().len());
            let mut v = Vec::with_capacity(cap);
            for rs in qr.result_sets {
                rs.to_rows().into_iter().fold(&mut v, |v, r| {
                    v.push(Either::Right(r));
                    v
                });
            }
            v
        }))))
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(self, query: E) -> BoxFuture<'e, Result<Option<YdbRow>, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Ydb> { Box::pin( async move {
        let rows = self.fetch_all(query).await?;
        Ok(rows.into_iter().next())
    })}

    fn prepare_with<'e, 'q: 'e>( self, sql: &'q str, parameters: &'e [YdbTypeInfo]) -> BoxFuture<'e, Result<YdbStatement, sqlx_core::Error>>
    where 'c: 'e {
        todo!()
    }

    fn describe<'e, 'q: 'e>(self, sql: &'q str) -> BoxFuture<'e, Result<Describe<Ydb>, sqlx_core::Error>>
    where 'c: 'e {
        todo!()
    }


    fn fetch_all<'e, 'q: 'e, E: 'q>( self, query: E ) -> BoxFuture<'e, Result<Vec<YdbRow>, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> {Box::pin ( async move {
        let result = self.execute(query).await?;
        let rs = result.result_sets.into_iter().next().unwrap();
        Ok(rs.to_rows())
    })}

    fn execute_many<'e, 'q: 'e, E: 'q>( self, query: E) -> BoxStream<'e, Result<YdbQueryResult, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> {
        Box::pin(self.execute(query).into_stream())
    }

    fn fetch<'e, 'q: 'e, E: 'q>(self, query: E) -> BoxStream<'e, Result<YdbRow, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> {
        Box::pin(Streamed::new(self.fetch_all(query)))
    }

    fn fetch_one<'e, 'q: 'e, E: 'q>(self, query: E) -> BoxFuture<'e, Result<YdbRow, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> { Box::pin( async move {
        let row = self.fetch_optional(query).await?;
        row.ok_or(sqlx_core::Error::RowNotFound)
    })}

    fn prepare<'e, 'q: 'e>(self, query: &'q str) -> BoxFuture<'e, Result<YdbStatement, sqlx_core::Error>>
    where 'c: 'e {
        self.prepare_with(query, &[])
    }

}

