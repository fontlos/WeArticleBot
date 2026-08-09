//! LLM AI 总结会话
//!
//! 通过 OpenAI 兼容的 chat/completions 接口做文章总结
//! 系统提示词暂留空, 后续再设计/配置
//!
//! 本模块暂未被业务调用, 统一允许 dead_code. 接入 AI 总结流程后应移除

#![allow(dead_code)]

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// 系统提示词
/// TODO: 待设计总结提示词
const SYSTEM_PROMPT: &str = "";

/// LLM 错误
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP 层错误
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    /// JSON 解析错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// API 返回非成功状态码
    #[error("LLM API error: {0}")]
    Api(String),
    /// 响应中缺少可用内容
    #[error("LLM response has no content: {0}")]
    Empty(String),
}

/// 结果别名
pub type Result<T> = std::result::Result<T, Error>;

/// LLM 会话, 持有 API 凭据与 HTTP 客户端
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

    /// 文章 AI 总结: 调用 {base_url}/chat/completions, 返回模型生成的总结文本
    pub async fn summarize(&self, text: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = ChatRequest {
            model: &self.model,
            messages: vec![
                Message {
                    role: "system",
                    content: SYSTEM_PROMPT,
                },
                Message {
                    role: "user",
                    content: text,
                },
            ],
        };

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(Error::Api(format!("{status}: {body_text}")));
        }

        let data: ChatResponse = resp.json().await?;
        data.choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| Error::Empty("no choices in response".to_owned()))
    }
}

/// chat/completions 请求体
#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
}

/// 对话消息
#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

/// chat/completions 响应体
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
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

    #[test]
    fn request_body_shape() {
        let req = ChatRequest {
            model: "model-x",
            messages: vec![
                Message {
                    role: "system",
                    content: SYSTEM_PROMPT,
                },
                Message {
                    role: "user",
                    content: "hello",
                },
            ],
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "model-x");
        assert_eq!(json["messages"][0]["role"], "system");
        assert_eq!(json["messages"][0]["content"], "");
        assert_eq!(json["messages"][1]["role"], "user");
        assert_eq!(json["messages"][1]["content"], "hello");
    }

    #[test]
    fn parse_chat_response() {
        let json = r#"{"choices":[{"message":{"content":"总结内容"}}]}"#;
        let data: ChatResponse = serde_json::from_str(json).unwrap();
        let content = data
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap();
        assert_eq!(content, "总结内容");
    }
}
