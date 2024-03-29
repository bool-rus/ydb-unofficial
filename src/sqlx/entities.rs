use std::sync::Arc;

use sqlx_core::HashMap;
use sqlx_core::type_info::TypeInfo;
use sqlx_core::value::ValueRef;
use sqlx_core::value::Value as XValue;
use sqlx_core::row::Row;
use sqlx_core::column::Column as XColumn;
use sqlx_core::column::ColumnIndex;
use ydb_grpc_bindings::generated::ydb;
use ydb::r#type::PrimitiveTypeId;
use ydb::value::Value;
use ydb::r#type::Type as YType;
use ydb::table_stats::QueryStats;
use ydb::Column;
use ydb::ResultSet;
use ydb::table::ExecuteQueryResult;

use super::database::Ydb;

#[derive(Debug, Clone)]
pub struct YdbValue {
    value: Value,
    info: YdbTypeInfo,
}

impl YdbValue {
    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl XValue for YdbValue {
    type Database = Ydb;
    fn as_ref(&self) -> YdbValueRef { &self }
    fn type_info(&self) -> std::borrow::Cow<'_, YdbTypeInfo> { std::borrow::Cow::Borrowed(&self.info) }
    fn is_null(&self) -> bool { matches!(self.value, Value::NullFlagValue(_)) }
}

pub type YdbValueRef<'a> = &'a YdbValue;

impl<'a> ValueRef<'a> for YdbValueRef<'a> {
    type Database = Ydb;
    fn to_owned(&self) -> YdbValue { Clone::clone(self) }
    fn type_info(&self) -> std::borrow::Cow<'_, YdbTypeInfo> { std::borrow::Cow::Borrowed(&self.info) }
    fn is_null(&self) -> bool { XValue::is_null(*self) }
}

#[derive(Debug, Clone, PartialEq)]
pub enum YdbTypeInfo {
    Primitive(PrimitiveTypeId),
    Null,
    Unknown,
}

impl Default for YdbTypeInfo {
    fn default() -> Self {
        Self::Unknown
    }
}
impl From<&ydb::OptionalType> for YdbTypeInfo {
    fn from(value: &ydb::OptionalType) -> Self {
        if let Some(t) = &value.item {
            if let Some(t) = &t.r#type {
                    return Self::from(t)
            }
        }
        Self::Unknown
    }
}
impl From<&YType> for YdbTypeInfo {
    fn from(value: &YType) -> Self {
        use YType::*;
        match value {
            TypeId(id) => Self::Primitive(PrimitiveTypeId::from_i32(*id).unwrap_or_default()),
            OptionalType(t) => Self::from(t.as_ref()),
            DecimalType(_) => todo!(),
            NullType(_) => Self::Null,
            _ => Self::Unknown
        }
    }
}

impl std::fmt::Display for YdbTypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(self.name())
    }
}

impl TypeInfo for YdbTypeInfo {
    fn is_null(&self) -> bool {
        matches!(&self, Self::Null)
    }
    fn name(&self) -> &str {
        match self {
            YdbTypeInfo::Primitive(t) => t.as_str_name(),
            YdbTypeInfo::Null => "NULL",
            YdbTypeInfo::Unknown => "UNKNOWN",
        }
    }
}

#[test]
fn sometest() {
    //let q: sqlx_core::query::Query = todo!();
}



#[derive(Debug, Clone, Default)]
pub struct YdbResultSet {
    columns: Arc<Columns>,
    rows: Vec<YdbRow>
}

#[derive(Debug, Clone, Default)]
pub struct YdbQueryResult {
    //TODO: добавить возврат updates, reads, deletes, affected_rows
    pub query_stats: Option<QueryStats>,
    pub result_sets: Vec<YdbResultSet>,
}

impl YdbResultSet {
    pub fn columns(&self) -> &[YdbColumn] {
        self.columns.columns.as_slice()
    }
    pub fn rows(&self) -> &[YdbRow] {
        &self.rows
    }
    pub fn to_rows(self) -> Vec<YdbRow> {
        self.rows
    }
}

#[derive(Debug, Default)]
struct Columns {
    map: sqlx_core::HashMap<String, usize>,
    columns: Vec<YdbColumn>,
}

impl Columns {
    fn new(columns: Vec<YdbColumn>) -> Arc<Self> {
        let map = columns.iter().fold(HashMap::new(), |mut map, col|{
            map.insert(col.name.to_owned(), col.ordinal);
            map
        });
        Arc::new(Self {map, columns})
    }
    fn as_slice(&self) -> &[YdbColumn] {
        &self.columns
    }
    fn get_index(&self, name: &str) -> Option<usize> {
        self.map.get(name).copied()
    }
    fn get(&self, idx: usize) -> Option<&YdbColumn> {
        self.columns.get(idx)
    }
    fn len(&self) -> usize {
        self.columns.len()
    }
}

impl From<ExecuteQueryResult> for YdbQueryResult {
    fn from(result: ExecuteQueryResult) -> Self {
        let ExecuteQueryResult {query_stats, result_sets, .. } = result;
        let result_sets = result_sets.into_iter().map(Into::into).collect();
        Self { query_stats, result_sets }
    }
}

impl From<ResultSet> for YdbResultSet {
    fn from(rs: ResultSet) -> Self {
        let ResultSet {columns, rows, ..} = rs;
        let columns = columns.into_iter().enumerate().map(YdbColumn::from).collect();
        let columns = Columns::new(columns);
        let rows = rows.into_iter().map(|row|YdbRow::create(columns.clone(), row)).collect();
        Self { columns, rows }
    }
}

impl Extend<Self> for YdbQueryResult {
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        for i in iter {
            self.result_sets.extend(i.result_sets);
            if let Some(qs) = &mut self.query_stats {
                if let Some(e) = i.query_stats {
                    qs.process_cpu_time_us += e.process_cpu_time_us;
                    qs.total_cpu_time_us += e.total_cpu_time_us;
                    qs.total_duration_us += e.total_duration_us;
                    //TODO: доработать extend QueryPhasesStats
                }
            } else {
                self.query_stats = i.query_stats;
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct YdbRow {
    columns: Arc<Columns>,
    row: Vec<YdbValue>,
}

impl YdbRow {
    fn create(columns: Arc<Columns>, row: ydb::Value) -> Self {
        let items = row.items;
        if items.len() != columns.len() {
            panic!("row len != columns len")
        }
        let row = items.into_iter().enumerate().map(|(i,value)|{
            let info = columns.get(i).unwrap().type_info.clone();
            let value = value.value.unwrap();
            YdbValue { value, info }
        }).collect();
        Self { columns, row }
    }
}

impl Row for YdbRow {
    type Database = Ydb;

    fn columns(&self) -> &[YdbColumn] {
        &self.columns.as_slice()
    }

    fn try_get_raw<I: ColumnIndex<Self>>(&self, index: I) -> Result<YdbValueRef, sqlx_core::Error> {
        let index = index.index(self)?;
        self.row.get(index).ok_or_else(|| sqlx_core::Error::ColumnIndexOutOfBounds { index, len: self.row.len() } )
    }
}

impl ColumnIndex<YdbRow> for &str {
    fn index(&self, row: &YdbRow) -> Result<usize, sqlx_core::Error> {
        row.columns.get_index(self)
        .ok_or_else(|| sqlx_core::Error::ColumnNotFound(self.to_string()) )
    }
}


#[derive(Debug, Clone)]
pub struct YdbColumn {
    pub(crate) ordinal: usize,
    pub(crate) name: String,
    pub(crate) type_info: YdbTypeInfo,
}

impl From<(usize, Column)> for YdbColumn {
    fn from((ordinal, c): (usize, Column)) -> Self {
        let Column { name, r#type } = c;
        let type_info = r#type.map(|t|t.r#type).flatten().map(|t|YdbTypeInfo::from(&t)).unwrap_or_default();
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
    println!("\nquery result: \n{qr:?}");
}
