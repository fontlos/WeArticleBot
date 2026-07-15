use lark::api::Message;
use log::{debug, info};

use std::time::Duration;

use crate::context;

pub async fn scan_login(chat_id: &str) {
    let lark = context::lark();
    let wechat = context::wechat();

    let msg = Message::to_chat(chat_id).text("正在获取微信登录二维码...");
    lark.send_message(msg).await.unwrap();

    let uuid = wechat.create_session().await.unwrap();
    info!("Created WeChat session with UUID: {}", uuid);

    let qrcode_bytes = wechat.get_qrcode().await.unwrap();

    let image_key = lark.upload_image(&qrcode_bytes).await.unwrap();

    let img = Message::to_chat(chat_id).image(&image_key);
    lark.send_message(img).await.unwrap();

    let interval = Duration::from_secs(2);
    let mut interval = tokio::time::interval(interval);

    loop {
        interval.tick().await;
        let status = wechat.check_qrcode().await.unwrap();
        debug!("QR code status: {}", status);
        if status == 1 {
            break;
        }
    }

    wechat.login().await.unwrap();
    let token = wechat.token();
    debug!("Login successful, token: {}", token);

    super::info::fetch_profile(chat_id).await;
}
