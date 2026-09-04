use serde::Deserialize;

use crate::session::Session;

use super::data::Res;

#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    /// 公众号唯一原始 ID, 用于本 crate API
    pub wxid: String,
    /// 公众号头像
    pub avatar: String,
    /// 公众号 ID, fakeid, 形如 `MzA4MDA0MzcwMA==`
    pub biz: String,
    /// 粉丝数
    pub follower_count: u32,
    /// 月平均阅读量
    pub month_read_avg: u32,
    /// 公众号昵称
    pub nickname: String,
    /// 公众号账号
    pub username: String,
}

impl Session {
    /// 获取公众号信息
    ///
    /// # Arguments
    /// * `biz` - 公众号 ID, fakeid, 形如 `MzA4MDA0MzcwMA==`
    /// * `nickname` - 公众号名称 (可选)
    ///
    /// **Cost: 0.04**
    pub async fn get_account_info(
        &self,
        biz: &str,
        nickname: Option<&str>,
    ) -> crate::Result<AccountInfo> {
        let url = "https://api.cimidata.com/api/v2/articles/current";
        self.refresh_token().await?;
        let token = self.token();

        // 构建请求体
        let json = serde_json::json!({
            "biz": biz,
            "nickname": nickname,
        });

        let query = [("access_token", token.as_str())];

        let bytes = self
            .client
            .post(url)
            .query(&query)
            .json(&json)
            .send()
            .await?
            .bytes()
            .await?;

        let res: AccountInfo = Res::parse(&bytes)?;

        Ok(res)
    }
}
