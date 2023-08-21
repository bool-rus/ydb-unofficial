use futures::StreamExt;
use futures::future::FutureExt;
use sqlx_core::describe::Describe;
use sqlx_core::executor::{Executor, Execute};
use sqlx_core::Either;
use tonic::codegen::futures_core::{future::BoxFuture, stream::BoxStream};
use ydb_grpc_bindings::generated::ydb;
use ydb::r#type::PrimitiveTypeId;
use ydb::table::{ExecuteDataQueryRequest, ExplainDataQueryRequest, PrepareDataQueryRequest, PrepareQueryResult};
use ydb_grpc_bindings::generated::ydb::table::ExecuteSchemeQueryRequest;

use crate::client::TableClientWithSession;
use crate::{YdbResponseWithResult, YdbTransaction};
use crate::error::YdbError;
use crate::auth::UpdatableToken;

use super::prelude::*;
type YdbExecutor<'c> = YdbTransaction<'c, UpdatableToken>;
type YdbSchemeExecutor<'c> = TableClientWithSession<'c, UpdatableToken>;

impl<'c> Executor<'c> for YdbExecutor<'c> {
    type Database = Ydb;

    fn execute<'e, 'q: 'e, E: 'q>(mut self, mut query: E,) -> BoxFuture<'e, Result<YdbQueryResult, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> {
        let parameters = query.take_arguments().map(|a|a.0).unwrap_or_default();
        let query = if let Some(statement) = query.statement() {
            Some(crate::generated::ydb::table::query::Query::Id(statement.query_id().to_owned()))
        } else {
            Some(crate::generated::ydb::table::query::Query::YqlText(query.sql().to_owned()))
        };
        let query = Some(crate::generated::ydb::table::Query{query});
        Box::pin(async move {
            let response = self.execute_data_query(ExecuteDataQueryRequest{ query, parameters, ..Default::default()}).await.unwrap();
            let result = response.into_inner().result().map_err(YdbError::from)?;
            Ok(result.into())
        })
    }

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<'e, Result<Either<YdbQueryResult, YdbRow>,sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Ydb> { 
        let stream = futures::stream::once(self.execute(query))
        .map(|r| {
            let mut err = Vec::with_capacity(1);
            let v = match r {
                Ok(v) => v.result_sets,
                Err(e) => {
                    err.push(Err(e));
                    vec![]
                },
            };
            let v = v.into_iter()
            .map(|rs|rs.to_rows().into_iter()).flatten()
            .map(|r|Ok(Either::Right(r)))
            .chain(err);
            futures::stream::iter(v)
        }).flatten();

        Box::pin(stream)
        
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(self, query: E) -> BoxFuture<'e, Result<Option<YdbRow>, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Ydb> { Box::pin( async move {
        let rows = self.fetch_all(query).await?;
        Ok(rows.into_iter().next())
    })}

    fn prepare<'e, 'q: 'e>(mut self, sql: &'q str) -> BoxFuture<'e, Result<YdbStatement, sqlx_core::Error>>
    where 'c: 'e {Box::pin(async move {
        let yql_text = sql.to_owned();
        let response = self.table_client().prepare_data_query(PrepareDataQueryRequest{yql_text, ..Default::default()}).await?;
        let PrepareQueryResult {query_id, parameters_types} = response.into_inner().result().map_err(YdbError::from)?;
        let parameters = parameters_types.into();
        let yql = sql.to_owned();
        Ok(YdbStatement {query_id, yql, parameters})
    })}

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
        let stream = futures::stream::once(self.fetch_all(query))
        .map(|r| {
            let mut err = Vec::with_capacity(1);
            let v = match r {
                Ok(v) => v,
                Err(e) => {
                    err.push(Err(e));
                    vec![]
                },
            };
            let v = v.into_iter().map(|i|Ok(i)).chain(err);
            futures::stream::iter(v)
        }).flatten();
        Box::pin(stream)
    }

    fn fetch_one<'e, 'q: 'e, E: 'q>(self, query: E) -> BoxFuture<'e, Result<YdbRow, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> { Box::pin( async move {
        let row = self.fetch_optional(query).await?;
        row.ok_or(sqlx_core::Error::RowNotFound)
    })}

    fn prepare_with<'e, 'q: 'e>(self, sql: &'q str, _parameters: &'e [YdbTypeInfo]) -> BoxFuture<'e, Result<YdbStatement, sqlx_core::Error>>
    where 'c: 'e { self.prepare(sql) }

    //TODO: спрятать под фичу
    fn describe<'e, 'q: 'e>(mut self, sql: &'q str) -> BoxFuture<'e, Result<Describe<Ydb>, sqlx_core::Error>>
    where 'c: 'e { Box::pin( async move {
        let response = self.table_client().explain_data_query(ExplainDataQueryRequest{ yql_text: sql.to_owned(), ..Default::default() }).await?;
        let result = response.into_inner().result().map_err(YdbError::from)?;
        let (_, mut node) = super::minikql::Node::parse(&result.query_ast).map_err(|_|YdbError::DecodeAst)?;
        node.eval();
        let outputs = super::minikql::invoke_outputs(&node).unwrap_or_default();
        let (columns, nullable) = outputs.into_iter().fold((vec![], vec![]), |(mut cols, mut nulls), (ordinal, name, typ, optional)|{
            nulls.push(Some(optional));
            let name = name.to_owned();
            let type_info = if let Some(t) = PrimitiveTypeId::from_str_name(&typ.to_ascii_uppercase()) {
                YdbTypeInfo::Primitive(t)
            } else {
                YdbTypeInfo::Unknown
            };
            cols.push(YdbColumn{ ordinal, name, type_info });
            (cols, nulls)
        });
        //TODO: implement parameters invoking
        let parameters = None;
        Ok(Describe { columns, parameters, nullable })
    })}
}


impl <'c> Executor<'c> for YdbSchemeExecutor<'c> {
    type Database = Ydb;

    fn execute<'e, 'q: 'e, E: 'q>(mut self, query: E,) -> BoxFuture<'e, Result<YdbQueryResult, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> {
        let yql_text = query.sql().to_owned();
        Box::pin(async move {
            self.execute_scheme_query(ExecuteSchemeQueryRequest{ yql_text, ..Default::default()}).await.unwrap();
            Ok(Default::default())
        })
    }
    fn execute_many<'e, 'q: 'e, E: 'q>( self, query: E) -> BoxStream<'e, Result<YdbQueryResult, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> {
        Box::pin(self.execute(query).into_stream())
    }
    fn fetch_many<'e, 'q: 'e, E: 'q>(self, _query: E,) -> BoxStream<'e, Result<Either<YdbQueryResult, YdbRow>, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> {
        Box::pin(futures::future::err(only_execute_err()).into_stream())
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(self, _query: E) -> BoxFuture<'e, Result<Option<YdbRow>, sqlx_core::Error>>
    where 'c: 'e, E: Execute<'q, Self::Database> {
        Box::pin(futures::future::err(only_execute_err()))
    }

    fn prepare_with<'e, 'q: 'e>( self, _sql: &'q str, _parameters: &'e [YdbTypeInfo]) -> BoxFuture<'e, Result<YdbStatement, sqlx_core::Error>>
    where 'c: 'e {
        Box::pin(futures::future::err(only_execute_err()))
    }

    fn describe<'e, 'q: 'e>(self, _sql: &'q str) -> BoxFuture<'e, Result<Describe<Ydb>, sqlx_core::Error>>
    where 'c: 'e { 
        Box::pin(futures::future::err(only_execute_err()))
    }
}

fn only_execute_err() -> sqlx_core::Error {
    sqlx_core::Error::AnyDriverError("Only execute method allowed in SchemeExecutor".to_owned().into())
}