use serde::Deserialize;

use crate::session::Session;

use super::data::Res;

#[derive(Debug, Deserialize)]
struct Balance {
    balance: u32,
}

impl Session {
    /// 查询余额, 单位: 分
    ///
    /// **Cost: 0**
    pub async fn get_balance(&self) -> crate::Result<u32> {
        let url = "https://api.cimidata.com/api/v2/user/balance";
        self.refresh_token().await?;
        let token = self.token();

        let query = [("access_token", token.as_str())];

        let bytes = self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .bytes()
            .await?;

        let res: Balance = Res::parse(&bytes)?;

        Ok(res.balance)
    }
}
