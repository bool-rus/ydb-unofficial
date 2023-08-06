use sqlx_core::encode::Encode;
use sqlx_core::{types::Type, decode::Decode};
use ydb_grpc_bindings::generated::ydb;
use ydb_grpc_bindings::generated::ydb::value::Value;
use ydb_grpc_bindings::generated::ydb::r#type::PrimitiveTypeId;

use super::YdbArgumentBuffer;

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
        use super::{Ydb, YdbTypeInfo, TypeKind, YdbValue};
        $(
        impl Type<Ydb> for $t {
            fn type_info() -> YdbTypeInfo {
                YdbTypeInfo::Normal(TypeKind::Primitive(PrimitiveTypeId::$info))
            }
        }
        
        impl Decode<'_, Ydb> for $t {
            fn decode(value: &YdbValue) -> Result<Self, sqlx_core::error::BoxDynError> {
                match value.value() {
                    Value::$val(v) => Ok(v.clone() as $t),
                    _ => Err(Box::new(AnotherType))
                }
            }
        }

        impl<'q, S: ToString + Clone> Encode<'q, Ydb> for (S, $t) {
            fn encode_by_ref(&self, buf: &mut YdbArgumentBuffer) -> sqlx_core::encode::IsNull {
                let value = ydb::Value {
                    value: Some(ydb::value::Value::$val(self.1.clone().into())),
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
}

