//! WS 客户端连接配置

use log::{error, warn};
use reqwest::Client;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use std::time::Duration;

/// 重试等待上限(10 分钟)
const MAX_BACKOFF: Duration = Duration::from_secs(10 * 60);

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
struct WsRes {
    code: i32,
    msg: String,
    data: Option<WsEndpoint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsEndpoint {
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(rename = "ClientConfig")]
    pub config: WsClientConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsClientConfig {
    /// 心跳间隔, 单位秒
    #[serde(rename = "PingInterval")]
    pub ping: i32,
    /// 重连次数, <=0 视为无限重试
    #[serde(rename = "ReconnectCount")]
    pub reconnect_count: i32,
    /// 重连基础间隔, 单位秒
    #[serde(rename = "ReconnectInterval")]
    pub reconnect_interval: i32,
    /// 重连随机抖动, 单位秒(预留)
    #[serde(rename = "ReconnectNonce")]
    pub reconnect_nonce: i32,
}

impl WsClientConfig {
    /// 计算第 attempt 次重试前应等待的时长
    pub fn backoff(&self, attempt: u32) -> Option<Duration> {
        let max_attempts = self.reconnect_count.max(0) as u32;
        if max_attempts > 0 && attempt >= max_attempts {
            return None;
        }

        // 指数退避: 基础间隔 * 2^(attempt-1)
        // 因子封顶 2^6; 上限不低于服务端基础间隔, 避免收敛过快导致重连过频
        let base = Duration::from_secs(self.reconnect_interval.max(1) as u64);
        let factor = 2u32.saturating_pow(attempt.saturating_sub(1).min(6));
        let cap = base.max(MAX_BACKOFF);
        let delay = base.saturating_mul(factor).min(cap);

        // 抖动: 基于尝试次数的确定性伪随机, 避免多实例同时重连
        let nonce_ms = (self.reconnect_nonce.max(0) as u64).saturating_mul(1000);
        let jitter = if nonce_ms > 0 {
            Duration::from_millis((attempt as u64 * 37) % nonce_ms)
        } else {
            Duration::ZERO
        };

        Some(delay + jitter)
    }

    /// 等待 backoff 计算出的时长, 并在等待期间响应停机信号
    pub async fn sleep_backoff(&self, attempt: u32, shutdown: &CancellationToken) -> bool {
        let Some(delay) = self.backoff(attempt) else {
            error!("Reconnect attempts exhausted, giving up");
            return false;
        };

        warn!(
            "WebSocket connection lost, retrying in {} seconds (attempt {})",
            delay.as_secs(),
            attempt
        );
        tokio::select! {
            _ = tokio::time::sleep(delay) => true,
            _ = shutdown.cancelled() => false,
        }
    }
}

// 获取 WebSocket 端点配置
pub async fn ws_endpoint(app_id: &str, app_secret: &str) -> Result<WsEndpoint> {
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
            "Bad Websocket endpoint config: code {}, message: {}",
            res.code, res.msg
        )));
    }

    match res.data {
        Some(data) => Ok(data),
        None => return Err(Error::Custom("No endpoint data".to_string())),
    }
}
