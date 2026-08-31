//! 公众号文章列表接口

use bytes::Bytes;
use serde::Deserialize;

use crate::session::Session;

use super::data::Res;

/// appmsgpublish 原始响应, 双层 JSON 字符串
#[derive(Debug, Deserialize)]
struct ListResponse {
    #[serde(deserialize_with = "deserialize_publish_page")]
    publish_page: PublishPage,
}

// 将 publish_page 的 JSON 字符串解析为 PublishPage,
// 空字符串(错误响应时)容忍为空列表,
// 这样 base_resp 的 ret != 0 错误能优先透出, 而不是被 JSON 解析错误掩盖
fn deserialize_publish_page<'de, D>(deserializer: D) -> Result<PublishPage, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(PublishPage {
            total_count: 0,
            publish_list: vec![],
        });
    }
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

// publish_page 解析后的结构
#[derive(Debug, Deserialize)]
struct PublishPage {
    total_count: usize,
    publish_list: Vec<PublishListItem>,
}

#[derive(Debug, Deserialize)]
struct PublishListItem {
    // 可能为空字符串(未发布的占位)或缺失, 统一过滤为 None
    #[serde(default, deserialize_with = "deserialize_publish_info")]
    publish_info: Option<PublishInfo>,
}

// 将 publish_info 的 JSON 字符串解析为 PublishInfo, 空字符串视为 None
fn deserialize_publish_info<'de, D>(deserializer: D) -> Result<Option<PublishInfo>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        serde_json::from_str(&s)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

// publish_info 解析后的结构
#[derive(Debug, Deserialize)]
struct PublishInfo {
    appmsgex: Vec<Article>,
}

/// 文章列表结果
#[derive(Debug)]
pub struct ArticleList {
    pub articles: Vec<Article>,
    /// 文章总数
    pub total: usize,
    /// 当前页没有文章即为加载完毕
    pub completed: bool,
}

/// 单篇公众号文章
#[derive(Debug, Deserialize)]
pub struct Article {
    /// 文章 ID
    pub aid: String,
    /// 群发消息 ID
    pub appmsgid: i64,
    /// 作者
    pub author_name: String,
    /// 封面图 URL
    pub cover: String,
    /// 发布时间(Unix 秒)
    pub create_time: i64,
    /// 摘要
    pub digest: String,
    /// 是否已删除
    pub is_deleted: bool,
    /// 同一群发消息中的序号
    pub itemidx: i32,
    /// 文章链接
    pub link: String,
    /// 标题
    pub title: String,
    /// 更新时间(Unix 秒)
    pub update_time: i64,
}

/// 从解析后的响应构建列表结果
fn build_list(resp: ListResponse) -> ArticleList {
    let articles: Vec<Article> = resp
        .publish_page
        .publish_list
        .into_iter()
        .filter_map(|item| item.publish_info)
        .flat_map(|info| info.appmsgex)
        .collect();
    let completed = articles.is_empty();
    ArticleList {
        articles,
        total: resp.publish_page.total_count,
        completed,
    }
}

impl Session {
    /// 获取公众号文章列表
    ///
    /// - fakeid: 公众号 ID
    /// - size: 每页数量
    /// - page: 页码, 从 1 开始
    /// - keyword: Some(关键词) 文章搜索模式, None 普通列表模式
    pub async fn list_articles(
        &self,
        id: &str,
        size: usize,
        page: usize,
        key: Option<&str>,
    ) -> crate::Result<ArticleList> {
        let url = "https://mp.weixin.qq.com/cgi-bin/appmsgpublish";
        let token = &self.token.load();
        let begin = (page - 1) * size;
        let is_search = key.is_some();

        let query = [
            ("sub", if is_search { "search" } else { "list" }),
            ("search_field", if is_search { "7" } else { "null" }),
            ("begin", &begin.to_string()),
            ("count", &size.to_string()),
            ("query", key.unwrap_or("")),
            ("fakeid", id),
            ("type", "101_1"),
            ("free_publish_type", "1"),
            ("sub_action", "list_ex"),
            ("token", token),
            ("lang", "zh_CN"),
            ("f", "json"),
            ("ajax", "1"),
        ];

        let bytes = self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .bytes()
            .await?;

        Ok(build_list(Res::parse(&bytes)?))
    }

    /// 抓取文章页面原始 HTML
    pub async fn fetch_article(&self, link: &str) -> crate::Result<Bytes> {
        let bytes = self.client.get(link).send().await?.bytes().await?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_response() -> String {
        // 构造与微信实际格式一致的响应: publish_page / publish_info 都是 JSON 字符串
        let publish_info = serde_json::json!({
            "appmsgex": [{
                "aid": "a1",
                "appmsgid": 123456789,
                "author_name": "测试作者",
                "cover": "http://example.com/cover.jpg",
                "create_time": 1700000000,
                "digest": "测试摘要",
                "is_deleted": false,
                "itemidx": 1,
                "link": "https://mp.weixin.qq.com/s?__biz=test&mid=1&idx=1&sn=abc",
                "title": "测试文章",
                "update_time": 1700000001,
            }]
        })
        .to_string();

        let publish_page = serde_json::json!({
            "total_count": 1,
            "publish_list": [{
                "publish_type": 1,
                "publish_info": publish_info,
            }]
        })
        .to_string();

        serde_json::json!({
            "base_resp": { "ret": 0, "err_msg": "ok" },
            "publish_page": publish_page,
        })
        .to_string()
    }

    #[test]
    fn parse_double_json_list() {
        let resp: ListResponse = Res::parse(sample_response().as_bytes()).unwrap();
        let list = build_list(resp);
        assert_eq!(list.total, 1);
        assert!(!list.completed);
        assert_eq!(list.articles.len(), 1);
        let article = &list.articles[0];
        assert_eq!(article.title, "测试文章");
        assert_eq!(article.author_name, "测试作者");
        assert_eq!(
            article.link,
            "https://mp.weixin.qq.com/s?__biz=test&mid=1&idx=1&sn=abc"
        );
        assert_eq!(article.appmsgid, 123456789);
    }

    #[test]
    fn parse_filters_empty_publish_info() {
        let publish_page = serde_json::json!({
            "total_count": 2,
            "publish_list": [
                { "publish_type": 1, "publish_info": "" },
                { "publish_type": 1, "publish_info": serde_json::json!({ "appmsgex": [] }).to_string() },
            ]
        })
        .to_string();
        let resp = serde_json::json!({
            "base_resp": { "ret": 0, "err_msg": "ok" },
            "publish_page": publish_page,
        });
        let resp: ListResponse = Res::parse(resp.to_string().as_bytes()).unwrap();
        let list = build_list(resp);
        assert!(list.articles.is_empty());
        assert!(list.completed);
        assert_eq!(list.total, 2);
    }

    #[test]
    fn parse_api_error_with_empty_publish_page() {
        // 错误响应时 publish_page 可能为空字符串, 应透出 base_resp 的错误
        let resp = serde_json::json!({
            "base_resp": { "ret": 200003, "err_msg": "session expired" },
            "publish_page": "",
        });
        let err = Res::<ListResponse>::parse(resp.to_string().as_bytes()).unwrap_err();
        assert!(err.to_string().contains("session expired"));
    }

    #[test]
    fn parse_empty_publish_page_ok() {
        // ret=0 但 publish_page 为空: 容忍为空列表
        let resp = serde_json::json!({
            "base_resp": { "ret": 0, "err_msg": "ok" },
            "publish_page": "",
        });
        let resp: ListResponse = Res::parse(resp.to_string().as_bytes()).unwrap();
        let list = build_list(resp);
        assert!(list.articles.is_empty());
        assert!(list.completed);
        assert_eq!(list.total, 0);
    }
}
