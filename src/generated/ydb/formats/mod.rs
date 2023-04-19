#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArrowBatchSettings {
    #[prost(bytes = "vec", tag = "1")]
    pub schema: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CsvSettings {
    /// Number of rows to skip before CSV data. It should be present only in the first upsert of CSV file.
    #[prost(uint32, tag = "1")]
    pub skip_rows: u32,
    /// Fields delimiter in CSV file. It's "," if not set.
    #[prost(bytes = "vec", tag = "2")]
    pub delimiter: ::prost::alloc::vec::Vec<u8>,
    /// String value that would be interpreted as NULL.
    #[prost(bytes = "vec", tag = "3")]
    pub null_value: ::prost::alloc::vec::Vec<u8>,
    /// First not skipped line is a CSV header (list of column names).
    #[prost(bool, tag = "4")]
    pub header: bool,
}
