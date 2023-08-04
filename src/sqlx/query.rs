use std::sync::Arc;

use sqlx_core::row::Row;
use sqlx_core::database::HasValueRef;
use sqlx_core::column::Column as XColumn;
use sqlx_core::column::ColumnIndex;
use ydb_grpc_bindings::generated::ydb;
use ydb::Column;
use ydb::ResultSet;
use ydb::table::ExecuteQueryResult;
use ydb::Value;

use super::YdbValue;
use super::YdbValueRef;
use super::{Ydb, YdbTypeInfo};


#[derive(Debug, Clone, Default)]
pub struct YdbQueryResult {
    columns: Arc<Vec<YdbColumn>>,
    rows: Vec<YdbRow>
}

#[test]
fn from_select_bots() {
    let bytes = include_bytes!("../../test/select_bots.protobytes");
    let result: ExecuteQueryResult = prost::Message::decode(bytes.as_slice()).unwrap(); 
    println!("val: {result:?}");
    for rs in &result.result_sets {
        println!("\n\n new result set ===========");
        println!("======columns: ");
        for col in &rs.columns {
            
            println!("{col:?}");
        }
        println!("\n======rows:");
        for r in &rs.rows {
            let r: Vec<_> = r.items.iter().map(|v|&v.value).collect();
            println!("{r:?}");
        }
    }
    let qr: YdbQueryResult = result.into();
}

impl From<ExecuteQueryResult> for YdbQueryResult {
    fn from(result: ExecuteQueryResult) -> Self {
        todo!()
    }
}

impl From<ResultSet> for YdbQueryResult {
    fn from(rs: ResultSet) -> Self {
        let ResultSet {columns, rows, ..} = rs;
        let columns = Arc::new(columns.into_iter().enumerate().map(YdbColumn::from).collect::<Vec<_>>());
        //let row
        todo!()
    }
}

impl Extend<Self> for YdbQueryResult {
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        log::error!("unimplemented")
    }
}

#[derive(Debug, Clone, Default)]
pub struct YdbRow {
    columns: Arc<Vec<YdbColumn>>,
    row: Vec<YdbValue>,
}

impl YdbRow {
    fn create(columns: Arc<Vec<YdbColumn>>, row: Value) -> Self {
        let items = row.items;
        if items.len() != columns.len() {
            panic!("row len != columns len")
        }
        items.into_iter().enumerate().map(|(i,value)|{
            let info = columns.get(i).unwrap().type_info.clone();
            let value = value.value.unwrap();
            YdbValue { value, info }
        });
        todo!()
    }
}

impl Row for YdbRow {
    type Database = Ydb;

    fn columns(&self) -> &[YdbColumn] {
        &self.columns
    }

    fn try_get_raw<I>(
        &self,
        index: I,
    ) -> Result<YdbValueRef, sqlx_core::Error>
    where
        I: ColumnIndex<Self> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct YdbColumn {
    ordinal: usize,
    name: String,
    type_info: YdbTypeInfo,
}

impl From<(usize, Column)> for YdbColumn {
    fn from((ordinal, c): (usize, Column)) -> Self {
        let Column { name, r#type } = c;
        let type_info = r#type.map(|t|t.r#type).flatten().map(Into::into).unwrap_or_default();
        Self {ordinal, name, type_info}
    }
}

impl XColumn for YdbColumn {
    type Database = Ydb;
    fn ordinal(&self) -> usize { self.ordinal }
    fn name(&self) -> &str { &self.name }
    fn type_info(&self) -> &YdbTypeInfo { &self.type_info }
}

sqlx_core::impl_column_index_for_row!{YdbRow}

