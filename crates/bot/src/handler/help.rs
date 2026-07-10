use lark::api::Message;

use crate::context;

pub async fn send_help(chat_id: &str) {
    let help_text = "命令提示
- help: 显示帮助信息
- info: 获取微信个人信息
- login: 获取微信登录二维码";

    let lark = context::lark();
    let msg = Message::to_chat(chat_id).text(help_text);
    lark.send_message(msg).await.unwrap();
}