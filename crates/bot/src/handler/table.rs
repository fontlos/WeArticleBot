//! 公众号数据写入多维表格

use lark::api::im::message::Message;
use serde_json::json;

use crate::context;
use crate::state::BitableState;

use super::search::AccountData;

/// add <index>: 把上次搜索结果中的第 index 个公众号写入「公众号」表
///
/// index 从 1 开始, 对应 search 命令输出列表的序号。
pub async fn add_account(chat_id: &str, index: usize) {
    if index == 0 {
        reply(chat_id, "索引从 1 开始").await;
        return;
    }

    let lark = context::lark();
    let bitable = lark.docs().bitable();

    let Some(state) = BitableState::load() else {
        reply(chat_id, "尚未初始化多维表格, 请先执行 init").await;
        return;
    };
    let Some(account) = super::search::get_account(index - 1) else {
        reply(chat_id, &format!("索引 {index} 无效, 请先执行 search")).await;
        return;
    };

    let fields = account_fields(&account);
    match bitable
        .create_record(&state.app_token, &state.accounts_table_id, &fields)
        .await
    {
        Ok(record) => {
            reply(
                chat_id,
                &format!("已添加: {} ({})", account.name, record.record_id),
            )
            .await;
        }
        Err(e) => reply(chat_id, &format!("添加失败: {e}")).await,
    }
}

/// 有什么字段就填什么, 其余交给 Bitable 默认值
fn account_fields(account: &AccountData) -> serde_json::Value {
    json!({
        "公众号名称": account.name,
        "fakeid": account.id,
        "头像": { "text": account.name, "link": account.head },
        "简介": account.signature,
        "抓取状态": "启用",
    })
}

async fn reply(chat_id: &str, text: &str) {
    let msg = Message::to_chat(chat_id).text(text);
    context::lark().send_message(msg).await.unwrap();
}
