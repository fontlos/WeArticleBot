//! 同步文章到多维表格

use lark::api::im::message::Message;
use serde_json::{Value, json};

use crate::context;
use crate::state::BitableState;

/// sync: 读取 公众号 表中的账号(wxid), 通过 cimi 获取历史文章并写入「文章」表
///
/// 暂不做去重与分页(测试阶段只有一个公众号); 正文由 summary 按需抓取
pub async fn sync_articles(chat_id: &str) {
    let lark = context::lark();
    let cimi = context::cimi();
    let bitable = lark.docs().bitable();

    let Some(state) = BitableState::load() else {
        reply(chat_id, "尚未初始化多维表格, 请先执行 init").await;
        return;
    };

    reply(chat_id, "正在读取公众号列表...").await;
    let accounts = match bitable
        .list_records(&state.app_token, &state.accounts_table_id, 100)
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
        let wxid = account
            .fields
            .get("wxid")
            .and_then(Value::as_str)
            .unwrap_or("");
        if wxid.is_empty() {
            errors.push(format!("{name}: 缺少 wxid, 请重新执行 add"));
            continue;
        }

        // cimi 历史文章(第一页)
        // TODO: last_id 循环翻页
        let page = match cimi.get_history_articles(wxid, None).await {
            Ok(page) => page,
            Err(e) => {
                errors.push(format!("{name}: 获取历史文章失败 {e}"));
                continue;
            }
        };

        let mut text = format!("公众号 '{}' 已获取 {} 篇文章:\n", name, page.items.len());
        for (i, article) in page.items.iter().enumerate() {
            text.push_str(&format!(
                "{}. {} ({})\n{}\n",
                i + 1,
                article.title,
                article.published_at,
                article.url
            ));
        }
        reply(chat_id, &text).await;

        let records: Vec<Value> = page
            .items
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

/// 有什么字段就填什么, 其余交给 Bitable 默认值; 正文由 summary 抓取后回填
fn article_record(account_record_id: &str, article: &cimi::Article) -> Value {
    let title = &article.title;
    let mut fields = json!({
        "标题": title,
        "公众号": [account_record_id],
        "摘要": article.digest,
        "封面": { "text": title, "link": article.cover },
        "原文链接": { "text": title, "link": article.url },
        "chksm": extract_chksm(&article.url),
        "处理状态": "待总结",
        // 正文: summary 时按需抓取
    });
    if let Some(ts) = parse_published_at(&article.published_at) {
        fields["发布时间"] = json!(ts);
    }
    fields
}

/// 从文章 URL 中截取 chksm 参数(去重键, 预留)
fn extract_chksm(url: &str) -> Option<String> {
    let query = url.split('?').nth(1)?;
    let query = query.split('#').next().unwrap_or(query);
    for pair in query.split('&') {
        if let Some(v) = pair.strip_prefix("chksm=") {
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// 解析 cimi 的发布时间字符串(UTC)为毫秒时间戳
///
/// 支持 "2024-05-16T14:03:54Z" 与 "2024-05-16 14:03:54" 形式; 解析失败返回 None
fn parse_published_at(s: &str) -> Option<i64> {
    let s = s.trim();
    let year: i64 = s.get(0..4)?.parse().ok()?;
    let month: i64 = s.get(5..7)?.parse().ok()?;
    let day: i64 = s.get(8..10)?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let (mut hour, mut minute, mut second) = (0i64, 0i64, 0i64);
    // 第 10 位是分隔符(T/空格), 第 11 位起是时间数字, 形如 "14:03:54Z"
    if let Some(rest) = s.get(11..) {
        if rest.len() >= 8 && rest.as_bytes().first().is_some_and(u8::is_ascii_digit) {
            hour = rest.get(0..2)?.parse().ok()?;
            minute = rest.get(3..5)?.parse().ok()?;
            second = rest.get(6..8)?.parse().ok()?;
            if hour > 23 || minute > 59 || second > 60 {
                return None;
            }
        }
    }

    let days = days_from_civil(year, month, day);
    let secs = days * 86400 + hour * 3600 + minute * 60 + second;
    Some(secs * 1000)
}

/// 公历日期 -> 自 1970-01-01 的天数(Howard Hinnant 算法)
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

async fn reply(chat_id: &str, text: &str) {
    let msg = Message::to_chat(chat_id).text(text);
    context::lark().send_message(msg).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_chksm_from_url() {
        let url = "https://mp.weixin.qq.com/s?__biz=MzA3NTY1MTUzOQ==&mid=1&idx=1&sn=abc&chksm=8567621cd9bbe762e77a9b2aee1d6c570575019d4953f51daef8a8042a5c337a5123481d8bd6&scene=27#wechat_redirect";
        assert_eq!(
            extract_chksm(url).as_deref(),
            Some("8567621cd9bbe762e77a9b2aee1d6c570575019d4953f51daef8a8042a5c337a5123481d8bd6")
        );
    }

    #[test]
    fn extract_chksm_missing() {
        assert_eq!(extract_chksm("https://mp.weixin.qq.com/s?__biz=x"), None);
    }

    #[test]
    fn parse_published_at_formats() {
        assert_eq!(
            parse_published_at("2024-05-16T14:03:54Z"),
            Some(1715868234000)
        );
        assert_eq!(
            parse_published_at("2024-05-16 14:03:54"),
            Some(1715868234000)
        );
        assert_eq!(parse_published_at("garbage"), None);
    }
}
