use arc_swap::ArcSwap;
use bytes::Bytes;
use log::error;
use reqwest::Client;
use tokio_util::sync::CancellationToken;

use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ws::{StopReason, WebSocketClient};

#[derive(Debug)]
pub struct Session {
    pub client: Client,
    pub app_id: String,
    pub app_secret: String,
    pub token: ArcSwap<String>,
    /// Token 有效时长, 最长 3 小时, 当剩余不到半小时时调用会刷新 token
    pub expire: AtomicU64,
}

impl Session {
    pub fn new(app_id: &str, app_secret: &str) -> Self {
        Session {
            client: Client::new(),
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            token: ArcSwap::default(),
            expire: AtomicU64::new(0),
        }
    }

    pub fn expire(&self) -> u64 {
        self.expire.load(Ordering::Acquire)
    }

    pub fn set_token(&self, token: String, expire: u64) {
        self.token.store(token.into());
        self.expire.store(expire, Ordering::Release);
    }

    /// 建立长连接并运行事件循环
    pub async fn run_ws<F, Fut>(&self, shutdown: CancellationToken, handler: F) -> crate::Result<()>
    where
        F: Fn(Bytes) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // 获取一次 endpoint 配置, 重连时复用
        let endpoint = WebSocketClient::get_endpoint(&self.app_id, &self.app_secret).await?;
        let mut attempt: u32 = 0;

        loop {
            if shutdown.is_cancelled() {
                return Ok(());
            }

            let reason = match WebSocketClient::connect_with_endpoint(endpoint.clone()).await {
                Ok(client) => client.run(shutdown.cancelled(), &handler).await,
                Err(e) => {
                    error!("WebSocket connect failed: {e}");
                    StopReason::ConnectionLost
                }
            };

            match reason {
                StopReason::Shutdown => return Ok(()),
                StopReason::ConnectionLost => {}
            }

            attempt += 1;
            if !endpoint.config.sleep_backoff(attempt, &shutdown).await {
                return Ok(());
            }
        }
    }
}
