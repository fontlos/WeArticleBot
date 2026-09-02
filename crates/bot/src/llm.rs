//! LLM AI 总结会话
//!
//! 通过 OpenAI 兼容的 chat/completions 接口做文章总结。
//! 使用 JSON Output 模式(response_format=json_object),
//! 返回四层结构化结果: 一句话总结 / 核心要点 / 关键数据 / 结论与启示。

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// 系统提示词: 四层提取, 只输出 JSON
const SYSTEM_PROMPT: &str = r#"你是一个专业的微信公众号文章总结助手。
请阅读用户提供的文章内容，提取信息，严格按以下四层结构输出 JSON：

1. one_line_summary: 一句话总结全文核心观点（不超过 60 字）
2. key_points: 核心要点列表（3-5 条，每条不超过 80 字）
3. key_data: 关键数据与事实（文章中的数字、时间、专有名词、结论性事实；没有就填空字符串）
4. conclusion: 结论与启示（作者想传达的结论或行动建议；没有就填空字符串）

只输出一个 JSON 对象，不要输出任何解释文字、markdown 代码块标记或其他内容。
JSON 结构示例：
{"one_line_summary":"一句话总结","key_points":["要点一","要点二"],"key_data":"关键数据","conclusion":"结论与启示"}"#;

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
    /// 其它错误(如返回内容不是预期 JSON)
    #[error("LLM error: {0}")]
    Custom(String),
}

/// 结果别名
pub type Result<T> = std::result::Result<T, Error>;

/// 四层结构化总结结果
#[derive(Debug, Deserialize)]
pub struct ArticleSummary {
    pub one_line_summary: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    #[serde(default)]
    pub key_data: String,
    #[serde(default)]
    pub conclusion: String,
}

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

    /// 文章 AI 总结: 调用 {base_url}/chat/completions, 返回四层结构化总结
    pub async fn summarize(&self, text: &str) -> Result<ArticleSummary> {
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
            response_format: ResponseFormat {
                r#type: "json_object",
            },
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
        let content = data
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| Error::Empty("no choices in response".to_owned()))?;

        parse_summary(&content)
    }
}

/// 解析模型输出的 JSON(容忍 ```json 代码块包裹)
fn parse_summary(content: &str) -> Result<ArticleSummary> {
    let content = content.trim();
    let content = content
        .strip_prefix("```json")
        .or_else(|| content.strip_prefix("```"))
        .map(|s| s.trim().trim_end_matches("```").trim())
        .unwrap_or(content);

    match serde_json::from_str::<ArticleSummary>(content) {
        Ok(summary) => Ok(summary),
        Err(e) => Err(Error::Custom(format!(
            "返回内容不是预期的 JSON: {e} ({})",
            &content[..content.len().min(200)]
        ))),
    }
}

/// chat/completions 请求体
#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    /// JSON 输出模式
    response_format: ResponseFormat,
}

/// 对话消息
#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

/// response_format: 固定 json_object
#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    r#type: &'static str,
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
    fn request_body_has_json_mode_and_prompt() {
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
            response_format: ResponseFormat {
                r#type: "json_object",
            },
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "model-x");
        assert_eq!(json["response_format"]["type"], "json_object");
        assert!(
            json["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("one_line_summary")
        );
        assert_eq!(json["messages"][1]["content"], "hello");
    }

    #[test]
    fn parse_structured_summary() {
        let content = r#"{"one_line_summary":"一句话","key_points":["要点1","要点2"],"key_data":"数据","conclusion":"结论"}"#;
        let s: ArticleSummary = serde_json::from_str(content).unwrap();
        assert_eq!(s.one_line_summary, "一句话");
        assert_eq!(s.key_points.len(), 2);
        assert_eq!(s.key_data, "数据");
        assert_eq!(s.conclusion, "结论");
    }

    #[test]
    fn parse_summary_tolerates_code_fence() {
        let content = "```json\n{\"one_line_summary\":\"a\",\"key_points\":[\"b\"]}\n```";
        let s = parse_summary(content).unwrap();
        assert_eq!(s.one_line_summary, "a");
        assert_eq!(s.key_points, vec!["b".to_string()]);
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
