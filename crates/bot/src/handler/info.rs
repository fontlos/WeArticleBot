use lark::api::Message;

use crate::context;

pub async fn fetch_profile(chat_id: &str) {
    let lark = context::lark();
    let wechat = context::wechat();

    wechat.set_token("724245888");

    let profile = wechat.fetch_profile().await.unwrap();

    let text = format!("当前登录用户: {}", profile.0);
    let msg = Message::to_chat(chat_id).text(&text);
    lark.send_message(msg).await.unwrap();
}
