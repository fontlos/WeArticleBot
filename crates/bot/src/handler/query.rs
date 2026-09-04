use lark::api::im::message::Message;
use lark::event::MessageEvent;

use crate::context;

pub async fn query_lark_profile(event: &MessageEvent) {
    let lark = context::lark();

    let userid = &event.sender.sender_id.open_id;

    let msg = Message::to_chat(event.chat_id()).text(&format!("Your user_id is: {}", userid));
    lark.send_message(msg).await.unwrap();
}

pub async fn query_wechat_profile(chat_id: &str) {
    let lark = context::lark();
    let wechat = context::wechat();

    let profile = wechat.fetch_profile().await.unwrap();

    let text = format!("当前登录用户: {}", profile.0);
    let msg = Message::to_chat(chat_id).text(&text);
    lark.send_message(msg).await.unwrap();
}
