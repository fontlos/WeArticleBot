//! 同步文章到多维表格

use lark::api::im::message::Message;
use serde_json::{Value, json};

use crate::context;
use crate::state::BitableState;

/// sync: 读取「公众号」表中的账号, 逐个列出文章并写入「文章」表
///
/// 暂不做去重与分页(测试阶段只有一个公众号)。
pub async fn sync_articles(chat_id: &str) {
    let lark = context::lark();
    let wechat = context::wechat();
    let bitable = lark.docs().bitable();

    let Some(state) = BitableState::load() else {
        reply(chat_id, "尚未初始化多维表格, 请先执行 init").await;
        return;
    };

    reply(chat_id, "正在读取公众号列表...").await;
    let accounts = match bitable
        .list_records(&state.app_token, &state.accounts_table_id, None, None, 100)
        .await
    {
        Ok(list) => list.items,
        Err(e) => {
            reply(chat_id, &format!("读取公众号列表失败: {e}")).await;
            return;
        }
    };
    if accounts.is_empty() {
        reply(chat_id, "公众号列表为空, 请先执行 add <index>").await;
        return;
    }

    let mut total = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for account in &accounts {
        let record_id = account.record_id.clone();
        let name = account
            .fields
            .get("公众号名称")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string();

        let articles = match wechat.list_articles().await {
            Ok(list) => list,
            Err(e) => {
                errors.push(format!("{name}: 获取文章失败 {e}"));
                continue;
            }
        };

        let records: Vec<Value> = articles
            .iter()
            .map(|a| article_record(&record_id, a))
            .collect();
        if records.is_empty() {
            continue;
        }

        match bitable
            .batch_create_records(&state.app_token, &state.articles_table_id, &records)
            .await
        {
            Ok(created) => total += created.len(),
            Err(e) => errors.push(format!("{name}: 写入失败 {e}")),
        }
    }

    if errors.is_empty() {
        reply(chat_id, &format!("同步完成, 共写入 {total} 篇文章")).await;
    } else {
        reply(
            chat_id,
            &format!(
                "同步完成(部分失败): 写入 {total} 篇, 错误: {}",
                errors.join("; ")
            ),
        )
        .await;
    }
}

/// 有什么字段就填什么, 其余交给 Bitable 默认值; 正文暂不抓取(预留)
fn article_record(account_record_id: &str, article: &wechat::Article) -> Value {
    let title = article.ext.title.clone();
    let link = decode_amp(&article.ext.content_url);
    json!({
        "标题": title,
        "公众号": [account_record_id],
        "摘要": article.ext.digest,
        "封面": { "text": title, "link": article.ext.cover },
        "原文链接": { "text": title, "link": link },
        "发布时间": article.comm.datetime * 1000,
        "appmsgid": article.comm.id.to_string(),
        "处理状态": "待总结",
        // 正文: 预留, 待 fetch_article 链路可用后填充
    })
}

/// content_url 里是 HTML 实体 &amp;, 链接使用前还原为 &
fn decode_amp(s: &str) -> String {
    s.replace("&amp;", "&")
}

async fn reply(chat_id: &str, text: &str) {
    let msg = Message::to_chat(chat_id).text(text);
    context::lark().send_message(msg).await.unwrap();
}
