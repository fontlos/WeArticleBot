use lark::api::im::message::Message;
use lark::event::MessageEvent;

use crate::context;

pub async fn query_user_id(event: &MessageEvent) {
    let lark = context::lark();

    let userid = &event.sender.sender_id.open_id;

    let msg = Message::to_chat(event.chat_id()).text(&format!("Your user_id is: {}", userid));
    lark.send_message(msg).await.unwrap();
}
