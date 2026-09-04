use serde::Deserialize;

use crate::session::Session;

use super::Res;

#[derive(Debug, Deserialize)]
pub struct Article {
    pub title: String,
    pub digest: String,
    #[serde(rename = "content_url")]
    pub url: String,
    pub cover: String,
    /// 发布时间(UTC)
    pub published_at: String,
}

impl Session {
    /// 获取公众号当天发文
    ///
    /// # Arguments
    /// * `wxid` - 公众号唯一原始 ID
    /// * `nickname` - 公众号名称（可选）
    pub async fn get_today_articles(
        &self,
        wxid: &str,
        nickname: Option<&str>,
    ) -> crate::Result<Vec<Article>> {
        let url = "https://api.cimidata.com/api/v2/articles/current";
        self.refresh_token().await?;
        let token = self.token();

        // 构建请求体
        let json = serde_json::json!({
            "wxid": wxid,
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

        let res: Vec<Article> = Res::parse(&bytes)?;

        Ok(res)
    }
}
