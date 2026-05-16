//! Protocol 结构定义

#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]
pub struct Header {
    #[prost(string, required, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Frame {
    #[prost(uint64, required, tag = "1")]
    pub seq_id: u64,
    #[prost(uint64, required, tag = "2")]
    pub log_id: u64,
    #[prost(int32, required, tag = "3")]
    pub service: i32,
    #[prost(int32, required, tag = "4")]
    pub method: i32,
    #[prost(message, repeated, tag = "5")]
    pub headers: ::prost::alloc::vec::Vec<Header>,
    #[prost(string, optional, tag = "6")]
    pub payload_encoding: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "7")]
    pub payload_type: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(bytes = "vec", optional, tag = "8")]
    pub payload: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(string, optional, tag = "9")]
    pub log_id_new: ::core::option::Option<::prost::alloc::string::String>,
}
