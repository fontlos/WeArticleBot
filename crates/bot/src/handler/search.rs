use lark::api::im::message::Message;

use crate::context;

pub async fn search_official(chat_id: &str, key: &str) {
    let lark = context::lark();
    let wechat = context::wechat();

    let msg = Message::to_chat(chat_id).text("正在搜索公众号...");
    lark.send_message(msg).await.unwrap();

    let result = wechat.search(key, 10, 1).await.unwrap();

    let mut text = format!("共 {} 个结果:\n", result.total);
    for (i, account) in result.list.iter().enumerate() {
        text.push_str(&format!(
            "{}. {} (ID: {})\n{}\n",
            i + 1,
            account.name,
            account.id,
            account.signature
        ));
    }

    let msg = Message::to_chat(chat_id).text(&text);
    lark.send_message(msg).await.unwrap();
}
