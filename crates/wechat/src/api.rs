use bytes::Bytes;

use crate::error::Result;
use crate::session::Session;
use crate::utils;

impl Session {
    pub async fn create_session(&self) -> Result<()> {
        let url = "https://mp.weixin.qq.com/cgi-bin/bizlogin?action=startlogin";
        let sid = utils::random_string(16);
        let form = [
            ("userlang", "zh_CN"),
            ("redirect_url", ""),
            ("login_type", "3"),
            ("sessionid", &sid),
            ("token", ""),
            ("lang", "zh_CN"),
            ("f", "json"),
            ("ajax", "1"),
        ];
        // {"base_resp":{"err_msg":"ok","ret":0},"uuid":""}
        let res = self.client.post(url).form(&form).send().await?;
        let text = res.text().await?;
        println!("Response: {}", text);
        Ok(())
    }

    /// 获取登录二维码, 返回二维码图片的 bytes, JPEG 格式
    pub async fn get_qrcode(&self) -> Result<Bytes> {
        let timestamp = utils::timestamp()?;
        let url = "https://mp.weixin.qq.com/cgi-bin/scanloginqrcode";
        let query = [("action", "getqrcode"), ("random", &timestamp.to_string())];
        let res = self.client.get(url).query(&query).send().await?;
        let bytes = res.bytes().await?;
        Ok(bytes)
    }
}
