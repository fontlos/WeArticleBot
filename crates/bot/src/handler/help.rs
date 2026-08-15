use lark::api::Message;

use crate::context;

pub async fn reply(chat_id: &str, text: &str) {
    let shell = format!("```shell\n{}\n```", text);
    let md = serde_json::json!({
        "zh_cn": {
            "title": "命令解析错误",
            "content": [
                [
                    {
                        "tag": "md",
                        "text": shell,
                    }
                ]
            ],
        }
    });
    let msg = Message::to_chat(chat_id).post(md.to_string());
    context::lark().send_message(msg).await.unwrap();
}
