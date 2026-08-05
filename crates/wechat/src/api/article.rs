//! 公众号文章列表接口

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::session::Session;

use super::data::BaseRes;

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

/// 文章列表结果
#[derive(Debug)]
pub struct ArticleList {
    pub articles: Vec<Article>,
    /// 文章总数(服务端返回)
    pub total: usize,
    /// 当前页没有文章即为加载完毕
    pub completed: bool,
}

/// appmsgpublish 外层响应: publish_page 是 JSON 字符串, 需要二次解析
#[derive(Debug, Deserialize)]
struct AppMsgPublishResponse {
    base_resp: BaseRes,
    publish_page: String,
}

/// publish_page 解析后的结构
#[derive(Debug, Deserialize)]
struct PublishPage {
    total_count: usize,
    publish_list: Vec<PublishListItem>,
}

#[derive(Debug, Deserialize)]
struct PublishListItem {
    /// 可能为空字符串(未发布的占位), 需要过滤
    publish_info: Option<String>,
}

/// publish_info 解析后的结构
#[derive(Debug, Deserialize)]
struct PublishInfo {
    appmsgex: Vec<Article>,
}

/// 每页文章数
const PAGE_SIZE: usize = 20;

impl Session {
    /// 获取公众号文章列表
    ///
    /// - fakeid: 公众号 ID
    /// - page: 页码, 从 1 开始
    /// - keyword: Some(关键词) 文章搜索模式, None 普通列表模式
    pub async fn list_articles(
        &self,
        fakeid: &str,
        page: usize,
        keyword: Option<&str>,
    ) -> Result<ArticleList> {
        let url = "https://mp.weixin.qq.com/cgi-bin/appmsgpublish";
        let token = &self.token.load();
        let begin = (page - 1) * PAGE_SIZE;
        let is_search = keyword.is_some();

        let query = [
            ("sub", if is_search { "search" } else { "list" }),
            ("search_field", if is_search { "7" } else { "null" }),
            ("begin", &begin.to_string()),
            ("count", &PAGE_SIZE.to_string()),
            ("query", keyword.unwrap_or("")),
            ("fakeid", fakeid),
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

        parse_response(&bytes)
    }
}

/// 解析响应(双层 JSON 字符串)
fn parse_response(bytes: &[u8]) -> Result<ArticleList> {
    let resp: AppMsgPublishResponse = serde_json::from_slice(bytes)?;
    if resp.base_resp.ret != 0 {
        return Err(Error::Custom(format!(
            "API error: {}",
            resp.base_resp.err_msg
        )));
    }

    // 第一层: publish_page(字符串) -> PublishPage
    let publish_page: PublishPage = serde_json::from_str(&resp.publish_page)?;

    // 第二层: publish_info(字符串) -> PublishInfo -> appmsgex
    let mut articles = Vec::new();
    for item in publish_page.publish_list {
        if let Some(info) = item.publish_info {
            if let Ok(info) = serde_json::from_str::<PublishInfo>(&info) {
                articles.extend(info.appmsgex);
            }
        }
    }

    Ok(ArticleList {
        completed: articles.is_empty(),
        total: publish_page.total_count,
        articles,
    })
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
        let list = parse_response(sample_response().as_bytes()).unwrap();
        assert_eq!(list.total, 1);
        assert!(!list.completed);
        assert_eq!(list.articles.len(), 1);
        let article = &list.articles[0];
        assert_eq!(article.title, "测试文章");
        assert_eq!(article.author_name, "测试作者");
        assert_eq!(article.link, "https://mp.weixin.qq.com/s?__biz=test&mid=1&idx=1&sn=abc");
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
        let list = parse_response(resp.to_string().as_bytes()).unwrap();
        assert!(list.articles.is_empty());
        assert!(list.completed);
        assert_eq!(list.total, 2);
    }

    #[test]
    fn parse_api_error() {
        let resp = serde_json::json!({
            "base_resp": { "ret": 200003, "err_msg": "session expired" },
            "publish_page": "",
        });
        let err = parse_response(resp.to_string().as_bytes()).unwrap_err();
        assert!(err.to_string().contains("session expired"));
    }
}
