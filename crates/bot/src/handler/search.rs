use lark::api::im::message::Message;
use std::sync::Mutex;

use crate::context;

/// 上一次搜索结果的暂存(内存)
static LAST_RESULTS: Mutex<Option<Vec<wechat::AccountInfo>>> = Mutex::new(None);

/// 从暂存中取第 index(0 基) 个公众号的字段快照
pub struct AccountData {
    pub name: String,
    pub id: String,
    pub head: String,
    pub signature: String,
}

pub fn get_account(index: usize) -> Option<AccountData> {
    let guard = LAST_RESULTS.lock().unwrap();
    let account = guard.as_ref()?.get(index)?;
    Some(AccountData {
        name: account.name.clone(),
        id: account.id.clone(),
        head: account.head.clone(),
        signature: account.signature.clone(),
    })
}

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

    // 暂存本次结果, 供 add <index> 使用
    *LAST_RESULTS.lock().unwrap() = Some(result.list);

    let msg = Message::to_chat(chat_id).text(&text);
    lark.send_message(msg).await.unwrap();
}
