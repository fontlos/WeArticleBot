use crate::error::Result;
use crate::session::Session;

impl Session {
    pub async fn search(&self) -> Result<()> {
        let url = "https://mp.weixin.qq.com/cgi-bin/searchbiz";
        let query = [
            ("action", "search_biz"),
            ("begin", "{BEGIN}"),
            ("count", "{SIZE}"),
            ("query", "{KEYWORD}"),
            ("token", "{TOKEN}"),
            ("lang", "zh_CN"),
            ("f", "json"),
            ("ajax", "1"),
        ];
        let res = self.client.get(url).query(&query).send().await?;
        println!("Url: {}", url);
        let text = res.text().await?;
        println!("Search response: {}", text);
        Ok(())
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
