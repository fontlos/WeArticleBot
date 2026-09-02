//! 公众号文章列表解析
//!
//! appmsgpublish 已被封禁, 数据源改为 profile_ext(home/getmsg)
//! 当前用演示数据 test.json 完成结构解析

use bytes::Bytes;
use serde::Deserialize;

use crate::session::Session;

#[derive(Debug, Deserialize)]
struct MsgPage {
    #[serde(deserialize_with = "deserialize_msg_list")]
    general_msg_list: Vec<Article>,
}

#[derive(Debug, Deserialize)]
struct GeneralMsgListWrap {
    list: Vec<Article>,
}

fn deserialize_msg_list<'de, D>(d: D) -> Result<Vec<Article>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    let wrap: GeneralMsgListWrap = serde_json::from_str(&s).map_err(serde::de::Error::custom)?;
    Ok(wrap.list)
}

/// 一条主页消息(可能含多图文)
#[derive(Debug, Deserialize)]
pub struct Article {
    #[serde(rename = "comm_msg_info")]
    pub comm: CommInfo,
    #[serde(rename = "app_msg_ext_info")]
    pub ext: ExtInfo,
}

/// 消息通用信息
#[derive(Debug, Deserialize)]
pub struct CommInfo {
    pub id: i64,
    pub r#type: i32,
    /// 发布时间(Unix 秒)
    pub datetime: i64,
    pub fakeid: String,
    pub status: i32,
    // pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtInfo {
    pub title: String,
    pub digest: String,
    // pub content: String,
    // pub fileid: i64,
    /// 文章链接(可能含 &amp; 实体, 使用前建议解码)
    pub content_url: String,
    // pub source_url: String,
    pub cover: String,
    // pub subtype: i32,
    // pub is_multi: i32,
    // pub author: String,
    // pub copyright_stat: i32,
    // pub duration: i64,
    // pub del_flag: i32,
    // pub item_show_type: i32,
    // pub audio_fileid: i64,
    // pub play_url: String,
    // pub malicious_title_reason_id: i32,
    // pub malicious_content_type: i32,
}

/// 规范化公众号文章链接:
/// 1. HTML 实体 &amp; -> &
/// 2. http:// 升级为 https:// (微信文章页需 https 才能正常抓取)
fn normalize_article_url(url: &str) -> String {
    let url = url.replace("&amp;", "&");
    if let Some(rest) = url.strip_prefix("http://") {
        format!("https://{rest}")
    } else {
        url
    }
}

impl Session {
    /// 获取公众号文章列表(演示: 读取根目录 test.json)
    pub async fn list_articles(&self) -> crate::Result<Vec<Article>> {
        let bytes = std::fs::read("./test.json").unwrap();
        let page: MsgPage = serde_json::from_slice(&bytes)?;
        let mut articles = page.general_msg_list;
        // 提取时预处理文章链接(&amp; 解码 + https 升级), 供后续 fetch_article 直接使用
        for article in &mut articles {
            article.ext.content_url = normalize_article_url(&article.ext.content_url);
        }
        Ok(articles)
    }

    /// 抓取文章页面原始 HTML
    pub async fn fetch_article(&self, link: &str) -> crate::Result<Bytes> {
        let bytes = self.client.get(link).send().await?.bytes().await?;
        Ok(bytes)
    }
}
