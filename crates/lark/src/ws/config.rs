use reqwest::Client;
use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
struct WsRes {
    code: i32,
    msg: String,
    data: Option<WsConfig>,
}

#[derive(Debug, Deserialize)]
pub struct WsConfig {
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "ClientConfig")]
    pub config: ClientConfig,
}

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    /// 心跳间隔, 单位秒
    #[serde(rename = "PingInterval")]
    pub ping: i32,
    // 重连次数
    // #[serde(rename = "ReconnectCount")]
    // reconnect_count: i32,
    // 重连间隔, 单位秒
    // #[serde(rename = "ReconnectInterval")]
    // reconnect_interval: i32,
    // 重连随机间隔, 单位秒
    // #[serde(rename = "ReconnectNonce")]
    // reconnect_nonce: i32,
}

// 获取 WebSocket 配置
pub async fn config(app_id: &str, app_secret: &str) -> Result<WsConfig> {
    let client = Client::new();
    let json = serde_json::json!({
        "AppID": app_id,
        "AppSecret": app_secret
    });
    let res = client
        .post("https://open.feishu.cn/callback/ws/endpoint")
        .json(&json)
        .send()
        .await?;

    let bytes = res.bytes().await?;
    let res: WsRes = serde_json::from_slice(&bytes)?;

    if res.code != 0 {
        return Err(Error::Custom(format!(
            "Bad Websocket config: code {}, message: {}",
            res.code, res.msg
        )));
    }

    match res.data {
        Some(data) => Ok(data),
        None => return Err(Error::Custom("No endpoint data".to_string())),
    }
}
