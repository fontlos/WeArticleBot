use lark::api::im::message::Message;
use log::debug;

use crate::context;

pub async fn search_official(chat_id: &str, key: &str) {
    let lark = context::lark();
    let wechat = context::wechat();

    let msg = Message::to_chat(chat_id).text("正在搜索公众号...");
    lark.send_message(msg).await.unwrap();

    let result = wechat.search(key, 1).await.unwrap();
    debug!("Search result: {:?}", result);

    let mut text = format!("共 {} 个结果:\n", result.total);
    for (i, account) in result.list.iter().enumerate() {
        text.push_str(&format!(
            "{}. {} (fakeid: {})\n{}",
            i + 1,
            account.nickname,
            account.fakeid,
            account.signature
        ));
    }

    let msg = Message::to_chat(chat_id).text(&text);
    lark.send_message(msg).await.unwrap();
}
