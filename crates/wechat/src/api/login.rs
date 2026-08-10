use bytes::Bytes;
use serde::Deserialize;

use crate::error::Error;
use crate::session::Session;
use crate::utils;

use super::data::Res;

impl Session {
    /// # 创建微信会话
    /// 返回会话的 UUID, 暂无主动用途
    pub async fn create_session(&self) -> crate::Result<String> {
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
        let bytes = self
            .client
            .post(url)
            .form(&form)
            .send()
            .await?
            .bytes()
            .await?;
        #[derive(Deserialize)]
        struct I {
            uuid: String,
        }
        let res: I = Res::parse(&bytes)?;
        Ok(res.uuid)
    }

    /// 获取登录二维码, 返回二维码图片的 bytes, JPEG 格式
    pub async fn get_qrcode(&self) -> crate::Result<Bytes> {
        let timestamp = utils::timestamp()?;
        let url = "https://mp.weixin.qq.com/cgi-bin/scanloginqrcode";
        let query = [("action", "getqrcode"), ("random", &timestamp.to_string())];
        let res = self.client.get(url).query(&query).send().await?;
        let bytes = res.bytes().await?;
        Ok(bytes)
    }

    /// # 检查二维码状态
    /// - `status=0`: 等待扫描
    /// - `status=1`: 扫码成功，继续登录
    /// - `status=2/3`: 二维码已失效，需刷新
    /// - `status=4/6`: 扫码成功，等待确认
    /// - `status=5`: 不支持扫码登录
    pub async fn check_qrcode(&self) -> crate::Result<i32> {
        let url = "https://mp.weixin.qq.com/cgi-bin/scanloginqrcode?action=ask&token=&lang=zh_CN&f=json&ajax=1";
        let bytes = self.client.get(url).send().await?.bytes().await?;
        #[derive(Deserialize)]
        struct I {
            status: i32,
        }
        let res: I = Res::parse(&bytes)?;
        Ok(res.status)
    }

    /// 继续完成登录
    pub async fn login(&self) -> crate::Result<()> {
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
        let bytes = self
            .client
            .post(url)
            .form(&form)
            .send()
            .await?
            .bytes()
            .await?;

        #[derive(Deserialize)]
        struct I {
            redirect_url: String,
        }
        let res: I = Res::parse(&bytes)?;
        // 第一步定位 token=, 第二步寻找&或直接匹配到末尾, 找不到要返回错误
        let token = res
            .redirect_url
            .split("token=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .ok_or_else(|| Error::Custom("Missing token in redirect URL".to_string()))?;
        self.set_token(token);
        Ok(())
    }
}
