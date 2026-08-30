use arc_swap::ArcSwap;
use bytes::Bytes;
use log::error;
use reqwest::Client;
use tokio_util::sync::CancellationToken;

use std::future::Future;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::ws::{StopReason, WebSocketClient};

/// 核心 API 分组标记
pub struct Core;

#[derive(Debug)]
pub struct Session<G = Core> {
    pub(crate) client: Client,
    pub(crate) app_id: String,
    pub(crate) app_secret: String,
    pub(crate) token: ArcSwap<String>,
    /// Token 有效时长, 最长 3 小时, 当剩余不到半小时时调用会刷新 token
    pub(crate) expire: AtomicU64,
    _marker: PhantomData<G>,
}

impl Session {
    pub fn new(app_id: &str, app_secret: &str) -> Self {
        Session {
            client: Client::new(),
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            token: ArcSwap::default(),
            expire: AtomicU64::new(0),
            _marker: PhantomData,
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

impl<G> Session<G> {
    /// Obtains a type-state view for the specified API group
    #[inline]
    pub const fn api<N>(&self) -> &Session<N> {
        unsafe {
            // Safety: PhantomData 不改变实际内存布局
            &*(self as *const Session<G> as *const Session<N>)
        }
    }

    /// 切换核心 API 分组实例
    pub fn core(&self) -> &Session<Core> {
        self.api()
    }
}
