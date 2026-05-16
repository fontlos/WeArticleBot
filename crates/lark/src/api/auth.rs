use crate::error::{Error, Result};
use crate::session::Session;
use crate::utils;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AccessToken {
    code: i32,
    msg: String,
    #[serde(rename = "tenant_access_token")]
    token: Option<String>,
    /// 过期时间, 每次调用重新计算, 最长 3 小时, 当剩余不到半小时时调用会刷新 token
    expire: Option<u64>,
}

impl Session {
    pub async fn refresh_access_token(&self) -> Result<()> {
        let now = utils::timestamp()?;
        if now < self.expire() {
            return Ok(());
        }
        let url = "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
        let json = serde_json::json!({
            "app_id": self.app_id,
            "app_secret": self.app_secret
        });
        let res = self
            .client
            .post(url)
            .json(&json)
            .send()
            .await?;
        let bytes = res.bytes().await?;
        let res = serde_json::from_slice::<AccessToken>(&bytes)?;
        if res.code != 0 {
            return Err(Error::Custom(format!(
                "Refresh access token error: code {}, message: {}",
                res.code, res.msg
            )));
        }
        match (res.token, res.expire) {
            (Some(token), Some(expire)) => {
                // 有效时长最长 3 小时, 当剩余不到半小时时调用会刷新 token, 所以我们少算十分钟
                self.set_token(token, expire - 600);
            }
            _ => return Err(Error::Custom("Invalid access token response".into())),
        }
        Ok(())
    }
}
