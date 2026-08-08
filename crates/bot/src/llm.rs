//! LLM AI 总结会话
//!
//! 目前仅预留接口与初始化逻辑, 尚未实现具体请求

#![allow(dead_code)]

use reqwest::Client;

/// LLM 错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// 接口尚未实现
    NotImplemented,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented => write!(f, "LLM summarize not implemented yet"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// LLM 会话
#[derive(Debug)]
pub struct Session {
    /// HTTP 客户端
    pub client: Client,
    /// API 基础地址
    pub base_url: String,
    /// API Key
    pub api_key: String,
    /// 模型名
    pub model: String,
}

impl Session {
    /// 从环境变量初始化
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Session {
            client: Client::new(),
            base_url: base_url.to_owned(),
            api_key: api_key.to_owned(),
            model: model.to_owned(),
        }
    }

    /// 文章 AI 总结
    pub async fn summarize(&self, _text: &str) -> Result<String> {
        Err(Error::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_new_keeps_credentials() {
        let session = Session::new("https://api.example.com", "token-123", "model-x");
        assert_eq!(session.base_url, "https://api.example.com");
        assert_eq!(session.api_key, "token-123");
        assert_eq!(session.model, "model-x");
    }
}
