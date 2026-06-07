use lark::api::Message;

pub async fn search_official(chat_id: &str, key: &str) {
    let lark = crate::lark();
    let wechat = crate::wechat();

    let msg = Message::to_chat(chat_id).text("正在搜索公众号...");
    lark.send_message(msg).await.unwrap();

    wechat.set_token("724245888");

    let res = wechat.search(key, 1).await.unwrap();
    println!("Search result: {}", String::from_utf8_lossy(&res));
}
