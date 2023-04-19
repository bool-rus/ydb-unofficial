#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FeatureFlag {}
/// Nested message and enum types in `FeatureFlag`.
pub mod feature_flag {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Status {
        Unspecified = 0,
        Enabled = 1,
        Disabled = 2,
    }
    impl Status {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Status::Unspecified => "STATUS_UNSPECIFIED",
                Status::Enabled => "ENABLED",
                Status::Disabled => "DISABLED",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STATUS_UNSPECIFIED" => Some(Self::Unspecified),
                "ENABLED" => Some(Self::Enabled),
                "DISABLED" => Some(Self::Disabled),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CostInfo {
    /// Total amount of request units (RU), consumed by the operation.
    #[prost(double, tag = "1")]
    pub consumed_units: f64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QuotaExceeded {
    #[prost(bool, tag = "1")]
    pub disk: bool,
}
/// Specifies a point in database time
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VirtualTimestamp {
    #[prost(uint64, tag = "1")]
    pub plan_step: u64,
    #[prost(uint64, tag = "2")]
    pub tx_id: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Limit {
    #[prost(oneof = "limit::Kind", tags = "1, 2, 3, 4, 5, 6")]
    pub kind: ::core::option::Option<limit::Kind>,
}
/// Nested message and enum types in `Limit`.
pub mod limit {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Range {
        #[prost(uint32, tag = "1")]
        pub min: u32,
        #[prost(uint32, tag = "2")]
        pub max: u32,
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Kind {
        #[prost(message, tag = "1")]
        Range(Range),
        #[prost(uint32, tag = "2")]
        Lt(u32),
        #[prost(uint32, tag = "3")]
        Le(u32),
        #[prost(uint32, tag = "4")]
        Eq(u32),
        #[prost(uint32, tag = "5")]
        Ge(u32),
        #[prost(uint32, tag = "6")]
        Gt(u32),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MapKey {
    #[prost(message, optional, tag = "1")]
    pub length: ::core::option::Option<Limit>,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusIds {}
/// Nested message and enum types in `StatusIds`.
pub mod status_ids {
    /// reserved range [400000, 400999]
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum StatusCode {
        Unspecified = 0,
        Success = 400000,
        BadRequest = 400010,
        Unauthorized = 400020,
        InternalError = 400030,
        Aborted = 400040,
        Unavailable = 400050,
        Overloaded = 400060,
        SchemeError = 400070,
        GenericError = 400080,
        Timeout = 400090,
        BadSession = 400100,
        PreconditionFailed = 400120,
        AlreadyExists = 400130,
        NotFound = 400140,
        SessionExpired = 400150,
        Cancelled = 400160,
        Undetermined = 400170,
        Unsupported = 400180,
        SessionBusy = 400190,
    }
    impl StatusCode {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                StatusCode::Unspecified => "STATUS_CODE_UNSPECIFIED",
                StatusCode::Success => "SUCCESS",
                StatusCode::BadRequest => "BAD_REQUEST",
                StatusCode::Unauthorized => "UNAUTHORIZED",
                StatusCode::InternalError => "INTERNAL_ERROR",
                StatusCode::Aborted => "ABORTED",
                StatusCode::Unavailable => "UNAVAILABLE",
                StatusCode::Overloaded => "OVERLOADED",
                StatusCode::SchemeError => "SCHEME_ERROR",
                StatusCode::GenericError => "GENERIC_ERROR",
                StatusCode::Timeout => "TIMEOUT",
                StatusCode::BadSession => "BAD_SESSION",
                StatusCode::PreconditionFailed => "PRECONDITION_FAILED",
                StatusCode::AlreadyExists => "ALREADY_EXISTS",
                StatusCode::NotFound => "NOT_FOUND",
                StatusCode::SessionExpired => "SESSION_EXPIRED",
                StatusCode::Cancelled => "CANCELLED",
                StatusCode::Undetermined => "UNDETERMINED",
                StatusCode::Unsupported => "UNSUPPORTED",
                StatusCode::SessionBusy => "SESSION_BUSY",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STATUS_CODE_UNSPECIFIED" => Some(Self::Unspecified),
                "SUCCESS" => Some(Self::Success),
                "BAD_REQUEST" => Some(Self::BadRequest),
                "UNAUTHORIZED" => Some(Self::Unauthorized),
                "INTERNAL_ERROR" => Some(Self::InternalError),
                "ABORTED" => Some(Self::Aborted),
                "UNAVAILABLE" => Some(Self::Unavailable),
                "OVERLOADED" => Some(Self::Overloaded),
                "SCHEME_ERROR" => Some(Self::SchemeError),
                "GENERIC_ERROR" => Some(Self::GenericError),
                "TIMEOUT" => Some(Self::Timeout),
                "BAD_SESSION" => Some(Self::BadSession),
                "PRECONDITION_FAILED" => Some(Self::PreconditionFailed),
                "ALREADY_EXISTS" => Some(Self::AlreadyExists),
                "NOT_FOUND" => Some(Self::NotFound),
                "SESSION_EXPIRED" => Some(Self::SessionExpired),
                "CANCELLED" => Some(Self::Cancelled),
                "UNDETERMINED" => Some(Self::Undetermined),
                "UNSUPPORTED" => Some(Self::Unsupported),
                "SESSION_BUSY" => Some(Self::SessionBusy),
                _ => None,
            }
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DecimalType {
    #[prost(uint32, tag = "1")]
    pub precision: u32,
    #[prost(uint32, tag = "2")]
    pub scale: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OptionalType {
    #[prost(message, optional, boxed, tag = "1")]
    pub item: ::core::option::Option<::prost::alloc::boxed::Box<Type>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListType {
    #[prost(message, optional, boxed, tag = "1")]
    pub item: ::core::option::Option<::prost::alloc::boxed::Box<Type>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VariantType {
    #[prost(oneof = "variant_type::Type", tags = "1, 2")]
    pub r#type: ::core::option::Option<variant_type::Type>,
}
/// Nested message and enum types in `VariantType`.
pub mod variant_type {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Type {
        #[prost(message, tag = "1")]
        TupleItems(super::TupleType),
        #[prost(message, tag = "2")]
        StructItems(super::StructType),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TupleType {
    #[prost(message, repeated, tag = "1")]
    pub elements: ::prost::alloc::vec::Vec<Type>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StructMember {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub r#type: ::core::option::Option<Type>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StructType {
    #[prost(message, repeated, tag = "1")]
    pub members: ::prost::alloc::vec::Vec<StructMember>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DictType {
    #[prost(message, optional, boxed, tag = "1")]
    pub key: ::core::option::Option<::prost::alloc::boxed::Box<Type>>,
    #[prost(message, optional, boxed, tag = "2")]
    pub payload: ::core::option::Option<::prost::alloc::boxed::Box<Type>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaggedType {
    #[prost(string, tag = "1")]
    pub tag: ::prost::alloc::string::String,
    #[prost(message, optional, boxed, tag = "2")]
    pub r#type: ::core::option::Option<::prost::alloc::boxed::Box<Type>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PgType {
    /// pg object id of the type
    /// full registry could be found here: <https://github.com/postgres/postgres/blob/master/src/include/catalog/pg_type.dat>
    ///
    /// required
    #[prost(uint32, tag = "1")]
    pub oid: u32,
    /// advanced type details useful for pg wire format proxying
    ///
    /// optional, set to 0 by default
    #[prost(int32, tag = "2")]
    pub typlen: i32,
    /// optional, set to 0 by default
    #[prost(int32, tag = "3")]
    pub typmod: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Type {
    #[prost(
        oneof = "r#type::Type",
        tags = "1, 2, 101, 102, 103, 104, 105, 106, 107, 201, 202, 203, 204, 205"
    )]
    pub r#type: ::core::option::Option<r#type::Type>,
}
/// Nested message and enum types in `Type`.
pub mod r#type {
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum PrimitiveTypeId {
        Unspecified = 0,
        Bool = 6,
        Int8 = 7,
        Uint8 = 5,
        Int16 = 8,
        Uint16 = 9,
        Int32 = 1,
        Uint32 = 2,
        Int64 = 3,
        Uint64 = 4,
        Float = 33,
        Double = 32,
        Date = 48,
        Datetime = 49,
        Timestamp = 50,
        Interval = 51,
        TzDate = 52,
        TzDatetime = 53,
        TzTimestamp = 54,
        String = 4097,
        Utf8 = 4608,
        Yson = 4609,
        Json = 4610,
        Uuid = 4611,
        JsonDocument = 4612,
        Dynumber = 4866,
    }
    impl PrimitiveTypeId {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                PrimitiveTypeId::Unspecified => "PRIMITIVE_TYPE_ID_UNSPECIFIED",
                PrimitiveTypeId::Bool => "BOOL",
                PrimitiveTypeId::Int8 => "INT8",
                PrimitiveTypeId::Uint8 => "UINT8",
                PrimitiveTypeId::Int16 => "INT16",
                PrimitiveTypeId::Uint16 => "UINT16",
                PrimitiveTypeId::Int32 => "INT32",
                PrimitiveTypeId::Uint32 => "UINT32",
                PrimitiveTypeId::Int64 => "INT64",
                PrimitiveTypeId::Uint64 => "UINT64",
                PrimitiveTypeId::Float => "FLOAT",
                PrimitiveTypeId::Double => "DOUBLE",
                PrimitiveTypeId::Date => "DATE",
                PrimitiveTypeId::Datetime => "DATETIME",
                PrimitiveTypeId::Timestamp => "TIMESTAMP",
                PrimitiveTypeId::Interval => "INTERVAL",
                PrimitiveTypeId::TzDate => "TZ_DATE",
                PrimitiveTypeId::TzDatetime => "TZ_DATETIME",
                PrimitiveTypeId::TzTimestamp => "TZ_TIMESTAMP",
                PrimitiveTypeId::String => "STRING",
                PrimitiveTypeId::Utf8 => "UTF8",
                PrimitiveTypeId::Yson => "YSON",
                PrimitiveTypeId::Json => "JSON",
                PrimitiveTypeId::Uuid => "UUID",
                PrimitiveTypeId::JsonDocument => "JSON_DOCUMENT",
                PrimitiveTypeId::Dynumber => "DYNUMBER",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "PRIMITIVE_TYPE_ID_UNSPECIFIED" => Some(Self::Unspecified),
                "BOOL" => Some(Self::Bool),
                "INT8" => Some(Self::Int8),
                "UINT8" => Some(Self::Uint8),
                "INT16" => Some(Self::Int16),
                "UINT16" => Some(Self::Uint16),
                "INT32" => Some(Self::Int32),
                "UINT32" => Some(Self::Uint32),
                "INT64" => Some(Self::Int64),
                "UINT64" => Some(Self::Uint64),
                "FLOAT" => Some(Self::Float),
                "DOUBLE" => Some(Self::Double),
                "DATE" => Some(Self::Date),
                "DATETIME" => Some(Self::Datetime),
                "TIMESTAMP" => Some(Self::Timestamp),
                "INTERVAL" => Some(Self::Interval),
                "TZ_DATE" => Some(Self::TzDate),
                "TZ_DATETIME" => Some(Self::TzDatetime),
                "TZ_TIMESTAMP" => Some(Self::TzTimestamp),
                "STRING" => Some(Self::String),
                "UTF8" => Some(Self::Utf8),
                "YSON" => Some(Self::Yson),
                "JSON" => Some(Self::Json),
                "UUID" => Some(Self::Uuid),
                "JSON_DOCUMENT" => Some(Self::JsonDocument),
                "DYNUMBER" => Some(Self::Dynumber),
                _ => None,
            }
        }
    }
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Type {
        /// Data types
        #[prost(enumeration = "PrimitiveTypeId", tag = "1")]
        TypeId(i32),
        #[prost(message, tag = "2")]
        DecimalType(super::DecimalType),
        /// Container types
        #[prost(message, tag = "101")]
        OptionalType(::prost::alloc::boxed::Box<super::OptionalType>),
        #[prost(message, tag = "102")]
        ListType(::prost::alloc::boxed::Box<super::ListType>),
        #[prost(message, tag = "103")]
        TupleType(super::TupleType),
        #[prost(message, tag = "104")]
        StructType(super::StructType),
        #[prost(message, tag = "105")]
        DictType(::prost::alloc::boxed::Box<super::DictType>),
        #[prost(message, tag = "106")]
        VariantType(super::VariantType),
        #[prost(message, tag = "107")]
        TaggedType(::prost::alloc::boxed::Box<super::TaggedType>),
        /// Special types
        #[prost(enumeration = "super::super::google::protobuf::NullValue", tag = "201")]
        VoidType(i32),
        #[prost(enumeration = "super::super::google::protobuf::NullValue", tag = "202")]
        NullType(i32),
        #[prost(enumeration = "super::super::google::protobuf::NullValue", tag = "203")]
        EmptyListType(i32),
        #[prost(enumeration = "super::super::google::protobuf::NullValue", tag = "204")]
        EmptyDictType(i32),
        #[prost(message, tag = "205")]
        PgType(super::PgType),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValuePair {
    #[prost(message, optional, tag = "1")]
    pub key: ::core::option::Option<Value>,
    #[prost(message, optional, tag = "2")]
    pub payload: ::core::option::Option<Value>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    /// Used for List, Tuple, Struct types
    #[prost(message, repeated, tag = "12")]
    pub items: ::prost::alloc::vec::Vec<Value>,
    /// Used for Dict type
    #[prost(message, repeated, tag = "13")]
    pub pairs: ::prost::alloc::vec::Vec<ValuePair>,
    /// Used for Variant type
    #[prost(uint32, tag = "14")]
    pub variant_index: u32,
    #[prost(fixed64, tag = "16")]
    pub high_128: u64,
    #[prost(oneof = "value::Value", tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 15")]
    pub value: ::core::option::Option<value::Value>,
}
/// Nested message and enum types in `Value`.
pub mod value {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(bool, tag = "1")]
        BoolValue(bool),
        #[prost(sfixed32, tag = "2")]
        Int32Value(i32),
        #[prost(fixed32, tag = "3")]
        Uint32Value(u32),
        #[prost(sfixed64, tag = "4")]
        Int64Value(i64),
        #[prost(fixed64, tag = "5")]
        Uint64Value(u64),
        #[prost(float, tag = "6")]
        FloatValue(f32),
        #[prost(double, tag = "7")]
        DoubleValue(f64),
        #[prost(bytes, tag = "8")]
        BytesValue(::prost::alloc::vec::Vec<u8>),
        #[prost(string, tag = "9")]
        TextValue(::prost::alloc::string::String),
        /// Set if current TValue is terminal Null
        #[prost(enumeration = "super::super::google::protobuf::NullValue", tag = "10")]
        NullFlagValue(i32),
        /// Represents nested TValue for Optional<Optional<T>>(Null), or Variant<T> types
        #[prost(message, tag = "11")]
        NestedValue(::prost::alloc::boxed::Box<super::Value>),
        #[prost(fixed64, tag = "15")]
        Low128(u64),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TypedValue {
    #[prost(message, optional, tag = "1")]
    pub r#type: ::core::option::Option<Type>,
    #[prost(message, optional, tag = "2")]
    pub value: ::core::option::Option<Value>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Column {
    /// Name of column
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Type of column
    #[prost(message, optional, tag = "2")]
    pub r#type: ::core::option::Option<Type>,
}
/// Represents table-like structure with ordered set of rows and columns
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResultSet {
    /// Metadata of columns
    #[prost(message, repeated, tag = "1")]
    pub columns: ::prost::alloc::vec::Vec<Column>,
    /// Rows of table
    #[prost(message, repeated, tag = "2")]
    pub rows: ::prost::alloc::vec::Vec<Value>,
    /// Flag indicates the result was truncated
    #[prost(bool, tag = "3")]
    pub truncated: bool,
}

pub mod discovery;

pub mod topic;

pub mod operations;

pub mod formats;
pub mod issue;
pub mod auth;
pub mod scheme;
pub mod table_stats;
pub mod table;