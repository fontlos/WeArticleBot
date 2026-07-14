use lark::api::Message;

use crate::context;

pub async fn reply(chat_id: &str, text: &str) {
    let msg = Message::to_chat(chat_id).text(text);
    context::lark().send_message(msg).await.unwrap();
}
