use serde::Deserialize;

use crate::session::Session;

use super::data::Res;

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

#[derive(Debug, Deserialize)]
pub struct ArticlePage {
    pub items: Vec<Article>,
    /// 下一页标识, 用于获取历史文章, 为空时表示没有更多文章
    pub last_id: Option<String>,
}

impl Session {
    /// 获取公众号当天发文
    ///
    /// # Arguments
    /// * `wxid` - 公众号唯一原始 ID
    /// * `nickname` - 公众号名称 (可选)
    ///
    /// **Cost: 0.04**
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

    /// 获取公众号当天发文
    ///
    /// # Arguments
    /// * `wxid` - 公众号唯一原始 ID
    /// * `last_id` - 下一页标识 (可选), 会在第一次请求时返回
    ///
    /// **Cost: 0.05**
    pub async fn get_history_articles(
        &self,
        wxid: &str,
        last_id: Option<&str>,
    ) -> crate::Result<ArticlePage> {
        let url = "https://api.cimidata.com/api/v2/articles/current";
        self.refresh_token().await?;
        let token = self.token();

        // 构建请求体
        let json = serde_json::json!({
            "wxid": wxid,
            "last_id": last_id,
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

        let res: ArticlePage = Res::parse(&bytes)?;

        Ok(res)
    }
}
