//! 初始化多维表格与权限

use lark::api::im::message::Message;
use lark::event::MessageEvent;

use crate::context;

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
    let new_bitable = bitable.create_bitable("WeArticleTable", &root).await.unwrap();

    let msg = Message::to_chat(event.chat_id()).text("正在初始化数据表...");
    lark.send_message(msg).await.unwrap();

    // 获取文件列表
    let files = drive.get_file_list().await.unwrap();
    let open_id = &event.sender.sender_id.open_id;

    // 初始化数据表
    let table = serde_json::json!({
        "table": {
            "name": "汇总表格",
            "default_view_name": "表格视图",
            "fields": [
                {
                    "field_name": "标题",
                    "type": 1
                },
                {
                    "field_name": "概述",
                    "type": 1
                },
                {
                    "field_name": "封面",
                    "type": 17
                },
                {
                    "field_name": "公众号",
                    "type": 1
                },
                {
                    "field_name": "原文链接",
                    "type": 15
                }
            ]
        }
    });

    let _table_res = bitable.create_table(&files.files[0], &table).await.unwrap();

    let msg = Message::to_chat(event.chat_id()).text(&format!("正在授予用户 {open_id} 多维表格管理权限..."));
    lark.send_message(msg).await.unwrap();

    // 授予用户编辑权限
    let _member = permission.add_member(&files.files[0], open_id).await.unwrap();

    let msg = Message::to_chat(event.chat_id()).text(&format!("已完成: {}", new_bitable.url));
    lark.send_message(msg).await.unwrap();
}

