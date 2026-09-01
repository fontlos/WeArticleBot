//! 搜索公众号接口

use serde::Deserialize;

use crate::session::Session;

use super::data::Res;

/// 搜索结果
#[derive(Debug, Deserialize)]
pub struct AccountList {
    pub list: Vec<AccountInfo>,
    pub total: usize,
}

/// 公众号信息
#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    /// 公众号 ID, 用于查询文章列表
    #[serde(rename = "fakeid")]
    pub id: String,
    /// 公众号名称
    #[serde(rename = "nickname")]
    pub name: String,
    /// 公众号账号
    pub alias: String,
    /// 头像 URL
    #[serde(rename = "round_head_img")]
    pub head: String,
    /// 公众号类型, 0: 订阅号, 1: 订阅号, 2: 服务号
    #[serde(rename = "service_type")]
    pub r#type: i32,
    /// 公众号签名简介
    pub signature: String,
}

impl Session {
    /// 搜索公众号
    pub async fn search(&self, key: &str, size: usize, page: usize) -> crate::Result<AccountList> {
        let url = "https://mp.weixin.qq.com/cgi-bin/searchbiz";
        let token = &self.token.load();
        let begin = (page - 1) * size;
        let query = [
            ("action", "search_biz"),
            ("begin", &begin.to_string()),
            ("count", &size.to_string()),
            ("query", key),
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

        let res: AccountList = Res::parse(&bytes)?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_search_response() {
        let json = serde_json::json!({
            "base_resp": { "ret": 0, "err_msg": "ok" },
            "list": [{
                "fakeid": "test_fakeid",
                "nickname": "test_nickname",
                "alias": "test_alias",
                "round_head_img": "http://example.com/head.jpg",
                "service_type": 0,
                "signature": "test_signature",
            }],
            "total": 1,
        });
        let _: AccountList = Res::parse(json.to_string().as_bytes()).unwrap();
    }
}
