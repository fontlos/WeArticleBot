use std::sync::OnceLock;

use bytes::Bytes;
use lark::api::Message;
use lark::event::{EventDispatcher, EventEnvelope, MessageEvent};

pub async fn handle(event: Bytes) {
    println!("Received event: {}", String::from_utf8_lossy(&event));

    if let Err(e) = dispatcher().dispatch(&event).await {
        eprintln!("事件处理失败: {}", e);
    }
}

fn dispatcher() -> &'static EventDispatcher {
    static DISPATCHER: OnceLock<EventDispatcher> = OnceLock::new();
    DISPATCHER.get_or_init(|| {
        let mut dispatcher = EventDispatcher::new();
        dispatcher.on("im.message.receive_v1", handle_message_event);
        dispatcher.fallback(handle_unsupported_event);
        dispatcher
    })
}

async fn handle_unsupported_event(envelope: EventEnvelope) -> lark::error::Result<()> {
    println!("未处理的事件类型: {}", envelope.event_type());
    Ok(())
}

async fn handle_message_event(envelope: EventEnvelope) -> lark::error::Result<()> {
    let msg_event = envelope.parse_event::<MessageEvent>()?;
    let chat_id = msg_event.chat_id();
    let text = msg_event.text().unwrap_or_default();
    let trimmed = text.trim();

    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let cmd = parts.next().unwrap_or_default();
    let _args = parts.next().unwrap_or_default();

    match cmd {
        "login" => send_login_qrcode(&chat_id).await,
        _ => send_help(&chat_id).await,
    }

    Ok(())
}

async fn send_help(chat_id: &str) {
    let session = crate::lark();
    let help_text = "命令提示
- help: 显示帮助信息
- login: 获取微信登录二维码";

    let msg = Message::to_chat(chat_id).text(help_text);
    session.send_message(msg).await.unwrap();
}

async fn send_login_qrcode(chat_id: &str) {
    let lark = crate::lark();
    let wechat = crate::wechat();

    let msg = Message::to_chat(chat_id).text("正在获取微信登录二维码...");
    lark.send_message(msg).await.unwrap();

    wechat.create_session().await.unwrap();
    let qrcode_bytes = wechat.get_qrcode().await.unwrap();

    let image_key = lark.upload_image(&qrcode_bytes).await.unwrap();

    let img = Message::to_chat(chat_id).image(&image_key);
    lark.send_message(img).await.unwrap();
}
