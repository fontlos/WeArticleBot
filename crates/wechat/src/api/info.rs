use crate::error::{Error, Result};
use crate::session::Session;
use crate::utils;

impl Session {
    // 主要用来测试登录状态
    pub async fn fetch_profile(&self) -> Result<(String, String)> {
        let url = "https://mp.weixin.qq.com/cgi-bin/home";
        let token = &self.token.load();
        let query = [
            ("t", "home/index"),
            ("token", token),
            ("lang", "zh_CN"),
        ];
        let resp = self.client
            .get(url)
            .query(&query)
            .send()
            .await?;

        let html = resp.bytes().await?;

        // 注意这两个字段周围的空格
        let nick_name = utils::parse_by_tag(&html, "wx.cgiData.nick_name = \"", "\"")
            .ok_or_else(|| Error::Custom("Failed to parse nick_name".into()))?
            .to_string();
        let head_img = utils::parse_by_tag(&html, "wx.cgiData.head_img = \"", "\"")
            .ok_or_else(|| Error::Custom("Failed to parse head_img".into()))?
            .to_string();

        Ok((nick_name, head_img))
    }
}
