use arc_swap::ArcSwap;
use reqwest::Client;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct Session {
    pub(crate) client: Client,
    pub(crate) app_id: String,
    pub(crate) app_secret: String,
    pub(crate) token: ArcSwap<String>,
    pub(crate) expire: AtomicU64,
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

    pub fn token(&self) -> Arc<String> {
        self.token.load().clone()
    }
}
