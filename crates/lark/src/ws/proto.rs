//! Protocol 结构定义

use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub struct Frame {
    // 序列号
    #[prost(uint64, required, tag = "1")]
    pub seq_id: u64,
    // 日志 ID, 旧的
    #[prost(uint64, required, tag = "2")]
    pub log_id: u64,
    // 服务号, 暂时不知道有什么用
    #[prost(int32, required, tag = "3")]
    pub service: i32,
    /// 方法, 目前已知 1 是有用的帧
    #[prost(int32, required, tag = "4")]
    pub method: i32,
    /// 下面这一条是有用的
    /// Header { key: "type", value: "event" }
    /// 目前已知, event 和空字符串需要处理, card 和其他情况忽略
    #[prost(message, repeated, tag = "5")]
    pub headers: Vec<Header>,
    /// 编码格式, 空字符串, 没什么用
    #[prost(string, optional, tag = "6")]
    pub payload_encoding: Option<String>,
    /// 编码类型, 空字符串
    #[prost(string, optional, tag = "7")]
    pub payload_type: Option<String>,
    /// 负载, JSON 数据
    #[prost(bytes = "vec", optional, tag = "8")]
    pub payload: Option<Vec<u8>>,
    /// 日志 ID, 新的, 没什么用
    #[prost(string, optional, tag = "9")]
    pub log_id_new: Option<String>,
}

impl Frame {
    pub fn response(&mut self, code: i32) {
        self.payload = Some(
            serde_json::json!({
                "code": code,
            })
            .to_string()
            .into_bytes(),
        );
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Message)]
pub struct Header {
    #[prost(string, required, tag = "1")]
    pub key: String,
    #[prost(string, required, tag = "2")]
    pub value: String,
}
