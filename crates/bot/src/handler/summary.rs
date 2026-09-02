//! 总结最早一篇待总结文章(仅测试行为)

use lark::api::im::message::Message;
use log::warn;
use serde_json::{Value, json};

use crate::context;
use crate::state::BitableState;

/// summary: 取文章表中「待总结」且发布时间最早的一篇,
/// 正文缺失时先抓取并回填, 再调 LLM 做四层总结,
/// 写入 AI总结表, 并把文章标记为已总结
pub async fn summarize_latest(chat_id: &str) {
    let lark = context::lark();
    let wechat = context::wechat();
    let bitable = lark.docs().bitable();

    let Some(state) = BitableState::load() else {
        reply(chat_id, "尚未初始化多维表格, 请先执行 init").await;
        return;
    };

    // 筛选: 处理状态 = 待总结; 排序: 发布时间正序(取最远/最早的一篇); 只取 1 条
    let filter = json!({
        "conjunction": "and",
        "conditions": [
            { "field_name": "处理状态", "operator": "is", "value": ["待总结"] }
        ]
    });
    let sort = json!([{ "field_name": "发布时间", "desc": false }]);

    let list = match bitable
        .search_records(
            &state.app_token,
            &state.articles_table_id,
            Some(&filter),
            Some(&sort),
            1,
        )
        .await
    {
        Ok(list) => list,
        Err(e) => {
            reply(chat_id, &format!("查询文章失败: {e}")).await;
            return;
        }
    };

    let Some(article) = list.items.into_iter().next() else {
        reply(chat_id, "没有待总结的文章").await;
        return;
    };
    let article_id = article.record_id;
    let field = |name: &str| {
        article
            .fields
            .get(name)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let title = field("标题");
    let digest = field("摘要");

    // 正文优先; 缺失时从「原文链接」抓取并回填到记录
    let body = field("正文");
    let body = if body.is_empty() {
        match fetch_and_store_body(wechat, &bitable, &state, &article_id, &article.fields).await {
            Some(md) => md,
            None => String::new(),
        }
    } else {
        body
    };

    let input = if !body.is_empty() {
        body
    } else if digest.is_empty() {
        title.clone()
    } else {
        format!("{title}\n{digest}")
    };
    if input.trim().is_empty() {
        reply(chat_id, "文章内容为空(正文抓取失败且无标题摘要), 无法总结").await;
        return;
    }

    reply(chat_id, "正在总结...").await;
    match context::llm().summarize(&input).await {
        Ok(summary) => {
            let model = context::llm().model.clone();
            let fields = json!({
                "一句话总结": summary.one_line_summary,
                "核心要点": summary.key_points.join("\n"),
                "关键数据": summary.key_data,
                "结论与启示": summary.conclusion,
                "关联文章": [article_id.clone()],
                "模型": model,
                "生成状态": "成功",
            });
            if let Err(e) = bitable
                .create_record(&state.app_token, &state.summaries_table_id, &fields)
                .await
            {
                reply(chat_id, &format!("写入总结失败: {e}")).await;
                return;
            }
            // 标记文章已总结
            let _ = bitable
                .update_record(
                    &state.app_token,
                    &state.articles_table_id,
                    &article_id,
                    &json!({ "处理状态": "已总结" }),
                )
                .await;
            reply(chat_id, "总结完成").await;
        }
        Err(e) => reply(chat_id, &format!("总结失败: {e}")).await,
    }
}

/// 从记录的「原文链接」抓取正文(HTML -> 提取 section -> markdown), 并回填到「正文」字段
async fn fetch_and_store_body(
    wechat: &wechat::Session,
    bitable: &lark::Session<lark::api::docs::Bitable>,
    state: &BitableState,
    article_id: &str,
    fields: &Value,
) -> Option<String> {
    let Some(link) = record_link(fields) else {
        return None;
    };

    let html = match wechat.fetch_article(&link).await {
        Ok(html) => html,
        Err(e) => {
            warn!("抓取正文失败: {e}");
            return None;
        }
    };
    let section = wechat::utils::extract_article_section(&html)?;
    let md = wechat::utils::article_to_markdown(section);
    if md.trim().is_empty() {
        return None;
    }

    // 回填正文, 供后续复用
    let _ = bitable
        .update_record(
            &state.app_token,
            &state.articles_table_id,
            article_id,
            &json!({ "正文": md }),
        )
        .await;
    Some(md)
}

/// 从记录的「原文链接」字段取 URL(超链接字段可能是字符串或 {"text","link"})
fn record_link(fields: &Value) -> Option<String> {
    let link = fields.get("原文链接")?;
    if let Some(s) = link.as_str() {
        return Some(s.to_string());
    }
    link.get("link").and_then(Value::as_str).map(str::to_string)
}

async fn reply(chat_id: &str, text: &str) {
    let msg = Message::to_chat(chat_id).text(text);
    context::lark().send_message(msg).await.unwrap();
}
