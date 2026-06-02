use arc_swap::ArcSwap;
use cookie_store::CookieStore;
use reqwest::Client;
use reqwest::header::{HeaderMap, ORIGIN, REFERER, USER_AGENT};
use reqwest_cookie_store::CookieStoreMutex;

use std::io::{BufRead, Write};
use std::sync::Arc;

use crate::error::Result;

const UA: &str = "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36 WAE/1.0";

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, UA.parse().unwrap());
    headers.insert(REFERER, "https://mp.weixin.qq.com/".parse().unwrap());
    headers.insert(ORIGIN, "https://mp.weixin.qq.com".parse().unwrap());
    headers
}

fn client(cookie: Arc<CookieStoreMutex>) -> Client {
    Client::builder()
        .cookie_provider(cookie)
        .default_headers(default_headers())
        .build()
        .unwrap()
}

#[derive(Debug)]
pub struct Session {
    pub(crate) client: Client,
    pub(crate) cookie: Arc<CookieStoreMutex>,
    pub(crate) token: ArcSwap<String>,
}

impl Session {
    pub fn new() -> Self {
        let cookie = Arc::new(CookieStoreMutex::new(CookieStore::default()));
        let client = client(cookie.clone());
        let token = ArcSwap::default();
        Self {
            client,
            cookie,
            token,
        }
    }

    pub fn token(&self) -> &ArcSwap<String> {
        &self.token
    }

    pub fn set_token(&self, token: &str) {
        self.token.store(Arc::new(token.to_owned()));
    }

    /// 仅用于测试
    #[allow(dead_code)]
    pub fn load<R: BufRead>(reader: R) -> Result<Self> {
        let cookie_store = CookieStore::load_all(reader, |s| serde_json::from_str(s))?;
        let cookie = Arc::new(CookieStoreMutex::new(cookie_store));
        let client = client(cookie.clone());
        let token = ArcSwap::default();
        Ok(Self {
            client,
            cookie,
            token,
        })
    }

    /// 仅用于测试
    #[allow(dead_code)]
    pub fn save<W: Write>(&self, writer: &mut W) -> Result<()> {
        let cookie_store = self.cookie.lock().unwrap();
        cookie_store.save_incl_expired_and_nonpersistent(writer, serde_json::to_string)?;
        Ok(())
    }
}
