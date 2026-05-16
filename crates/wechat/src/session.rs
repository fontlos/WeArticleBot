use cookie_store::CookieStore;
use reqwest::Client;
use reqwest::header::{HeaderMap, ORIGIN, REFERER, USER_AGENT};
use reqwest_cookie_store::CookieStoreMutex;

use std::io::{BufRead, Write};
use std::sync::Arc;

use crate::error::Result;

const UA: &str = "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36 WAE/1.0";

fn default_headers() -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, UA.parse()?);
    headers.insert(REFERER, "https://mp.weixin.qq.com/".parse()?);
    headers.insert(ORIGIN, "https://mp.weixin.qq.com".parse()?);
    Ok(headers)
}

pub struct Session {
    pub client: Client,
    pub cookie_store: Arc<CookieStoreMutex>,
}

impl Session {
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        let cookie_store = Arc::new(CookieStoreMutex::new(CookieStore::default()));
        let headers = default_headers()?;
        let client = Client::builder()
            .cookie_provider(cookie_store.clone())
            .default_headers(headers)
            .build()?;
        Ok(Self {
            client,
            cookie_store,
        })
    }

    /// 仅用于测试
    #[allow(dead_code)]
    pub fn load<R: BufRead>(reader: R) -> Result<Self> {
        let cookie_store = CookieStore::load_all(reader, |s| serde_json::from_str(s))?;
        let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));
        let client = Client::builder()
            .cookie_provider(cookie_store.clone())
            .default_headers(default_headers()?)
            .build()?;
        Ok(Self {
            client,
            cookie_store,
        })
    }

    /// 仅用于测试
    #[allow(dead_code)]
    pub fn save<W: Write>(&self, writer: &mut W) -> Result<()> {
        let cookie_store = self.cookie_store.lock().unwrap();
        cookie_store.save_incl_expired_and_nonpersistent(writer, serde_json::to_string)?;
        Ok(())
    }
}
