use bytes::Bytes;

use crate::error::Result;
use crate::session::Session;

impl Session {
    pub async fn search(&self, key: &str, page: usize) -> Result<Bytes> {
        let url = "https://mp.weixin.qq.com/cgi-bin/searchbiz";
        let token = &self.token.load();
        let size = 10;
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
        let res = self.client.get(url).query(&query).send().await?;
        let bytes = res.bytes().await?;
        Ok(bytes)
    }

    pub async fn list(&self) -> Result<()> {
        let url = "https://mp.weixin.qq.com/cgi-bin/appmsgpublish";
        let query = [
            ("sub", "list"),
            ("search_field", "null"),
            ("begin", "{begin}"),
            ("count", "{size}"),
            ("query", ""),
            ("fakeid", "{fakeid}"),
            ("type", "101_1"),
            ("free_publish_type", "1"),
            ("sub_action", "list_ex"),
            ("token", "{TOKEN}"),
            ("lang", "zh_CN"),
            ("f", "json"),
            ("ajax", "1"),
        ];
        let res = self.client.get(url).query(&query).send().await?;
        println!("Url: {}", url);
        let text = res.text().await?;
        println!("List response: {}", text);
        Ok(())
    }
}
