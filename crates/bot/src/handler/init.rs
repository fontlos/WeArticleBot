//! 初始化多维表格与权限

use lark::api::im::message::Message;
use lark::event::MessageEvent;
use serde_json::Value;
use serde_json::json;

use crate::context;

/// 表 1: 公众号
fn accounts_table() -> Value {
    json!({
        "table": {
            "name": "公众号",
            "default_view_name": "表格视图",
            "fields": [
                { "field_name": "公众号名称", "type": 1 },
                { "field_name": "fakeid", "type": 1 },
                { "field_name": "wxid", "type": 1 },
                { "field_name": "头像", "type": 15 },
                { "field_name": "简介", "type": 1 },
                {
                    "field_name": "抓取状态",
                    "type": 3,
                    "property": {
                        "options": [
                            { "name": "启用", "color": 10 },
                            { "name": "停用", "color": 0 }
                        ]
                    }
                },
                {
                    "field_name": "文章总数",
                    "type": 2,
                    "property": { "formatter": "0" }
                },
                {
                    "field_name": "已同步文章数",
                    "type": 2,
                    "property": { "formatter": "0" }
                },
                {
                    "field_name": "最后同步日期",
                    "type": 5,
                    "property": { "date_formatter": "yyyy-MM-dd HH:mm" }
                },
                { "field_name": "最后同步标题", "type": 1 },
                {
                    "field_name": "最近抓取时间",
                    "type": 5,
                    "property": { "date_formatter": "yyyy-MM-dd HH:mm" }
                },
                {
                    "field_name": "最近抓取结果",
                    "type": 3,
                    "property": {
                        "options": [
                            { "name": "成功", "color": 10 },
                            { "name": "失败", "color": 0 }
                        ]
                    }
                }
            ]
        }
    })
}

/// 表 2: 文章
fn articles_table(accounts_table_id: &str) -> Value {
    json!({
        "table": {
            "name": "文章",
            "default_view_name": "表格视图",
            "fields": [
                { "field_name": "标题", "type": 1 },
                {
                    "field_name": "公众号",
                    "type": 21,
                    "property": {
                        "table_id": accounts_table_id,
                        "back_field_name": "文章列表",
                        "multiple": true
                    }
                },
                { "field_name": "摘要", "type": 1 },
                { "field_name": "封面", "type": 15 },
                { "field_name": "原文链接", "type": 15 },
                { "field_name": "作者", "type": 1 },
                {
                    "field_name": "发布时间",
                    "type": 5,
                    "property": { "date_formatter": "yyyy-MM-dd HH:mm" }
                },
                { "field_name": "chksm", "type": 1 },
                { "field_name": "正文", "type": 1 },
                {
                    "field_name": "处理状态",
                    "type": 3,
                    "property": {
                        "options": [
                            { "name": "待总结", "color": 20 },
                            { "name": "已总结", "color": 10 },
                            { "name": "失败", "color": 0 }
                        ]
                    }
                }
            ]
        }
    })
}

/// 表 3: AI总结
fn summaries_table(articles_table_id: &str) -> Value {
    json!({
        "table": {
            "name": "AI总结",
            "default_view_name": "表格视图",
            "fields": [
                { "field_name": "一句话总结", "type": 1 },
                {
                    "field_name": "关联文章",
                    "type": 21,
                    "property": {
                        "table_id": articles_table_id,
                        "back_field_name": "AI总结",
                        "multiple": true
                    }
                },
                { "field_name": "核心要点", "type": 1 },
                { "field_name": "关键数据", "type": 1 },
                { "field_name": "结论与启示", "type": 1 },
                {
                    "field_name": "生成状态",
                    "type": 3,
                    "property": {
                        "options": [
                            { "name": "成功", "color": 10 },
                            { "name": "失败", "color": 0 }
                        ]
                    }
                },
                {
                    "field_name": "生成时间",
                    "type": 1001,
                    "property": { "date_formatter": "yyyy-MM-dd HH:mm" }
                },
                { "field_name": "模型", "type": 1 },
                { "field_name": "提示词版本", "type": 1 }
            ]
        }
    })
}

pub async fn init_bitable(event: &MessageEvent) {
    let lark = context::lark();

    // 获取各 API 组
    let drive = lark.docs().drive();
    let bitable = lark.docs().bitable();
    let permission = lark.docs().permission();

    // 获取根文件夹
    let root = drive.get_root_folder().await.unwrap();

    let msg = Message::to_chat(event.chat_id()).text("正在创建多维表格...");
    lark.send_message(msg).await.unwrap();

    // 创建多维表格
    let new_bitable = bitable
        .create_bitable("WeArticleTable", &root)
        .await
        .unwrap();
    let app_token = &new_bitable.app_token;

    let msg = Message::to_chat(event.chat_id()).text("正在初始化数据表...");
    lark.send_message(msg).await.unwrap();

    // 按依赖顺序建表
    let accounts = bitable
        .create_table(app_token, &accounts_table())
        .await
        .unwrap();
    let articles = bitable
        .create_table(app_token, &articles_table(&accounts.table_id))
        .await
        .unwrap();
    let summaries = bitable
        .create_table(app_token, &summaries_table(&articles.table_id))
        .await
        .unwrap();

    // 保存表格定位信息, 供 add 等命令复用
    crate::state::BitableState {
        app_token: new_bitable.app_token.clone(),
        accounts_table_id: accounts.table_id,
        articles_table_id: articles.table_id,
        summaries_table_id: summaries.table_id,
    }
    .save();

    // 删除自带默认表
    bitable
        .delete_table(app_token, &new_bitable.default_table_id)
        .await
        .unwrap();

    let open_id = &event.sender.sender_id.open_id;

    let msg = Message::to_chat(event.chat_id())
        .text(&format!("正在授予用户 {open_id} 多维表格管理权限..."));
    lark.send_message(msg).await.unwrap();

    // 授予用户编辑权限
    let _member = permission
        .add_member(app_token, "bitable", open_id)
        .await
        .unwrap();

    let msg = Message::to_chat(event.chat_id()).text(&format!("已完成: {}", new_bitable.url));
    lark.send_message(msg).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_tables_payload_shape() {
        // 表 1: 公众号
        let accounts = accounts_table();
        assert_eq!(accounts["table"]["name"], "公众号");
        let accounts_fields = accounts["table"]["fields"].as_array().unwrap();
        assert_eq!(accounts_fields.first().unwrap()["field_name"], "公众号名称");
        for name in ["文章总数", "已同步文章数", "最后同步日期", "最后同步标题"]
        {
            let f = accounts_fields
                .iter()
                .find(|f| f["field_name"] == name)
                .unwrap();
            assert!(f["type"].as_i64().is_some(), "{name} 缺失");
        }
        let total = accounts_fields
            .iter()
            .find(|f| f["field_name"] == "文章总数")
            .unwrap();
        assert_eq!(total["type"].as_i64(), Some(2));

        // 表 2: 文章, 公众号关联内联在第 2 位
        let articles = articles_table("tbl_accounts");
        assert_eq!(articles["table"]["name"], "文章");
        let article_fields = articles["table"]["fields"].as_array().unwrap();
        assert_eq!(article_fields.first().unwrap()["field_name"], "标题");
        let link = &article_fields[1];
        assert_eq!(link["field_name"], "公众号");
        assert_eq!(link["type"].as_i64(), Some(21));
        assert_eq!(link["property"]["table_id"], "tbl_accounts");
        assert_eq!(link["property"]["back_field_name"], "文章列表");
        assert_eq!(link["property"]["multiple"], true);

        // 表 3: AI总结
        let summaries = summaries_table("tbl_articles");
        assert_eq!(summaries["table"]["name"], "AI总结");
        let summaries_fields = summaries["table"]["fields"].as_array().unwrap();
        assert_eq!(
            summaries_fields.first().unwrap()["field_name"],
            "一句话总结"
        );
        let link = &summaries_fields[1];
        assert_eq!(link["field_name"], "关联文章");
        assert_eq!(link["type"].as_i64(), Some(21));
        assert_eq!(link["property"]["table_id"], "tbl_articles");
        assert_eq!(link["property"]["back_field_name"], "AI总结");
        assert_eq!(link["property"]["multiple"], true);
        for name in ["核心要点", "关键数据", "结论与启示"] {
            let f = summaries_fields
                .iter()
                .find(|f| f["field_name"] == name)
                .unwrap();
            assert!(f["type"].as_i64().is_some(), "{name} 缺失");
        }

        for f in article_fields
            .iter()
            .chain(accounts_fields.iter())
            .chain(summaries_fields.iter())
        {
            if f["type"].as_i64() == Some(15) {
                assert!(f.get("property").is_none(), "type 15 不能带 property: {f}");
            }
        }
    }
}