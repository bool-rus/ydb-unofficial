#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListEndpointsRequest {
    #[prost(string, tag = "1")]
    pub database: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub service: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EndpointInfo {
    /// This is an address (usually fqdn) and port of this node's grpc endpoint
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(uint32, tag = "2")]
    pub port: u32,
    #[prost(float, tag = "3")]
    pub load_factor: f32,
    #[prost(bool, tag = "4")]
    pub ssl: bool,
    #[prost(string, repeated, tag = "5")]
    pub service: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, tag = "6")]
    pub location: ::prost::alloc::string::String,
    #[prost(uint32, tag = "7")]
    pub node_id: u32,
    /// Optional ipv4 and/or ipv6 addresses of the endpoint, which clients may
    /// use instead of a dns name in the address field.
    #[prost(string, repeated, tag = "8")]
    pub ip_v4: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag = "9")]
    pub ip_v6: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Optional value for grpc.ssl_target_name_override option that must be
    /// used when connecting to this endpoint. This may be specified when an ssl
    /// endpoint is using certificate chain valid for a balancer hostname, and
    /// not this specific node hostname.
    #[prost(string, tag = "10")]
    pub ssl_target_name_override: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListEndpointsResult {
    #[prost(message, repeated, tag = "1")]
    pub endpoints: ::prost::alloc::vec::Vec<EndpointInfo>,
    #[prost(string, tag = "2")]
    pub self_location: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListEndpointsResponse {
    #[prost(message, optional, tag = "1")]
    pub operation: ::core::option::Option<super::operations::Operation>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WhoAmIRequest {
    /// Include user groups in response
    #[prost(bool, tag = "1")]
    pub include_groups: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WhoAmIResult {
    /// User SID (Security ID)
    #[prost(string, tag = "1")]
    pub user: ::prost::alloc::string::String,
    /// List of group SIDs (Security IDs) for the user
    #[prost(string, repeated, tag = "2")]
    pub groups: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WhoAmIResponse {
    #[prost(message, optional, tag = "1")]
    pub operation: ::core::option::Option<super::operations::Operation>,
}

pub mod v1;
