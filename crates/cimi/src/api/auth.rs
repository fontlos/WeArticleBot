use serde::Deserialize;

use crate::error::Error;
use crate::session::Session;
use crate::utils;

use super::Res;

#[derive(Debug, Deserialize)]
struct AccessToken {
    #[serde(rename = "access_token")]
    token: Option<String>,
}

impl Session {
    /// 刷新 access token
    pub async fn refresh_token(&self) -> crate::Result<()> {
        let now = utils::timestamp()?;
        if now < self.expire() {
            return Ok(());
        }
        let url = "https://api.cimidata.com/api/v2/token";
        let json = serde_json::json!({
            "app_id": self.app_id,
            "app_secret": self.app_secret
        });
        let res = self.client.post(url).json(&json).send().await?;
        let bytes = res.bytes().await?;
        println!("refresh_token response: {}", String::from_utf8_lossy(&bytes));
        let res: AccessToken = Res::parse(&bytes)?;

        match res.token {
            Some(token) => {
                // 有效时长最长 7 天, 提前 10 分钟刷新
                let expire = now + 7 * 24 * 60 * 60;
                self.set_token(token, expire - 600);
            }
            _ => return Err(Error::Custom("Invalid access token response".into())),
        }
        Ok(())
    }
}

