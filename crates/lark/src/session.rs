use arc_swap::ArcSwap;
use reqwest::Client;

use std::sync::atomic::{AtomicU64, Ordering};

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
}
