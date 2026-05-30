use bytes::Bytes;
use serde::Deserialize;

use crate::error::Result;
use crate::session::Session;
use crate::utils;

use super::data::Res;

impl Session {
    pub async fn create_session(&self) -> Result<String> {
        #[derive(Deserialize)]
        struct I {
            uuid: String,
        }

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
        let bytes = self.client.post(url).form(&form).send().await?.bytes().await?;
        let res: I = Res::parse(&bytes)?;
        Ok(res.uuid)
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

    /// 检查二维码状态
    // - `status=0`：等待扫描
    // - `status=1`：扫码成功，继续登录
    // - `status=2/3`：二维码已失效，需刷新
    // - `status=4/6`：扫码成功，等待确认
    // - `status=5`：不支持扫码登录
    pub async fn check_qrcode(&self) -> Result<i32> {
        #[derive(Deserialize)]
        struct I {
            status: i32,
        }

        let url = "https://mp.weixin.qq.com/cgi-bin/scanloginqrcode?action=ask&token=&lang=zh_CN&f=json&ajax=1";
        let bytes = self.client.get(url).send().await?.bytes().await?;
        println!("Url: {}", url);
        let res: I = Res::parse(&bytes)?;
        println!("Check QR code response: {}", res.status);
        Ok(res.status)
    }

    /// 继续完成登录
    pub async fn login(&self) -> Result<()> {
        let url = "https://mp.weixin.qq.com/cgi-bin/bizlogin?action=login";
        let form = [
            ("userlang", "zh_CN"),
            ("redirect_url", ""),
            ("cookie_forbidden", "0"),
            ("cookie_cleaned", "0"),
            ("plugin_used", "0"),
            ("login_type", "3"),
            ("token", ""),
            ("lang", "zh_CN"),
            ("f", "json"),
            ("ajax", "1"),
        ];
        let res = self.client.post(url).form(&form).send().await?;
        println!("Url: {}", url);
        let text = res.text().await?;
        println!("Login response: {}", text);
        Ok(())
    }

    pub async fn login_check(&self) -> Result<()> {
        let url = "https://mp.weixin.qq.com/cgi-bin/home";
        let query = [("t", "home/index"), ("token", "{TOKEN}"), ("lang", "zh_CN")];
        let res = self.client.get(url).query(&query).send().await?;
        println!("Url: {}", url);
        let text = res.text().await?;
        println!("Login check response: {}", text);
        Ok(())
    }
}
