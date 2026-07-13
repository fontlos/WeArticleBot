use lark::api::Message;

use crate::command;
use crate::context;

/// 发送帮助信息; name 为 None 时列出全部命令
pub async fn send_help(chat_id: &str, name: Option<&str>) {
    let text = match name {
        Some(name) => match command::command_help(name) {
            Some(help) => help,
            None => command::unknown_text(name),
        },
        None => command::general_help(),
    };

    let msg = Message::to_chat(chat_id).text(&text);
    if let Err(e) = context::lark().send_message(msg).await {
        eprintln!("发送消息失败: {e}");
    }
}

pub async fn reply(chat_id: &str, text: &str) {
    let msg = Message::to_chat(chat_id).text(text);
    if let Err(e) = context::lark().send_message(msg).await {
        eprintln!("发送消息失败: {e}");
    }
}
