use sqlx_core::{types::Type, decode::Decode, encode::Encode};
use ydb_grpc_bindings::generated::ydb::{self, value::Value, r#type::PrimitiveTypeId};
use super::database::{Ydb, YdbArgumentBuffer};
use super::entities::{YdbValue, YdbTypeInfo};

#[derive(Debug)]
struct AnotherType;
impl std::fmt::Display for AnotherType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
impl std::error::Error for AnotherType {}

macro_rules! ydb_type {
    ($($t:ty = ($info:ident, $val:ident),)+) => {
        $(
        impl Type<Ydb> for $t {
            fn type_info() -> YdbTypeInfo {
                YdbTypeInfo::Primitive(PrimitiveTypeId::$info)
            }
        }
        
        impl Decode<'_, Ydb> for $t {
            fn decode(value: &YdbValue) -> Result<Self, sqlx_core::error::BoxDynError> {
                match value.value() {
                    Value::$val(v) => Ok(v.clone().try_into().unwrap()),
                    _ => Err(Box::new(AnotherType))
                }
            }
        }

        impl<S: ToString + Clone> Type<Ydb> for (S, $t) {
            fn type_info() -> YdbTypeInfo {
                YdbTypeInfo::Primitive(PrimitiveTypeId::$info)
            }
        }
        
        impl<'q, S: ToString + Clone> Encode<'q, Ydb> for (S, $t) {
            fn encode_by_ref(&self, buf: &mut YdbArgumentBuffer) -> sqlx_core::encode::IsNull {
                let value = ydb::Value {
                    value: Some(ydb::value::Value::$val(self.1.clone().try_into().unwrap())),
                    ..Default::default()
                };
                let value = ydb::TypedValue {
                    r#type: Some(ydb::Type{r#type: Some(ydb::r#type::Type::TypeId(PrimitiveTypeId::$info.into()))}),
                    value: Some(value),
                };
                buf.insert(self.0.to_string(), value);
                sqlx_core::encode::IsNull::No
            }
        }
    )+}
}


macro_rules! wrapper_types {
    ($($t:ident ($inner:ty) $fun:ident ),+) => {$(
        
        #[derive(Debug, Clone, Copy)]
        pub struct $t($inner);
        impl Into<$inner> for $t { fn into(self) -> $inner {self.0} }
        impl From<$inner> for $t { fn from(v: $inner) -> Self {Self(v)} }
        impl $t {pub fn $fun(&self) -> $inner { self.0 }}
    )+};
}
wrapper_types! {
    Date(u16) days, 
    Datetime(u32) secs, 
    Timestamp(u64) micros, 
    Interval(i64) micros
}

impl Into<u32> for Date { fn into(self) -> u32 { self.0.into() }}
impl From<u32> for Date { fn from(v: u32) -> Self {Self(v.try_into().unwrap())}}


ydb_type! {
    bool = (Bool, BoolValue),
    i8  = (Int8, Int32Value),
    u8  = (Uint8, Uint32Value),
    i16 = (Int16, Int32Value),
    u16 = (Uint16, Uint32Value),
    i32 = (Int32, Int32Value),
    u32 = (Uint32, Uint32Value),
    i64 = (Int64, Int64Value),
    u64 = (Uint64, Uint64Value),
    f32 = (Float, FloatValue),
    f64 = (Double, DoubleValue),
    Vec<u8> = (String, BytesValue),
    String = (Utf8, TextValue),
    Date = (Date, Uint32Value),
    Datetime = (Datetime, Uint32Value),
    Timestamp = (Timestamp, Uint64Value),
}

