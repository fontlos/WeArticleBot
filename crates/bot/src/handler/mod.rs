mod help;
mod info;
mod login;
mod search;

use bytes::Bytes;

use lark::event::{EventDispatcher, EventEnvelope, MessageEvent};

use std::sync::OnceLock;

use crate::command;

pub async fn handle(event: Bytes) {
    println!("Received event: {}", String::from_utf8_lossy(&event));

    if let Err(e) = dispatcher().dispatch(&event).await {
        eprintln!("事件处理失败: {}", e);
    }
}

/// 事件分发器
fn dispatcher() -> &'static EventDispatcher {
    static DISPATCHER: OnceLock<EventDispatcher> = OnceLock::new();
    DISPATCHER.get_or_init(|| {
        let mut dispatcher = EventDispatcher::new();
        dispatcher.fallback(handle_unsupported_event);
        dispatcher.on("im.message.receive_v1", handle_message_event);
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

    match command::parse(&text) {
        Ok(parsed) => match parsed.kind {
            command::Kind::Help => {
                help::send_help(chat_id, parsed.args.first().map(String::as_str)).await
            }
            command::Kind::Info => info::fetch_profile(chat_id).await,
            command::Kind::Login => login::scan_login(chat_id).await,
            command::Kind::Search => search::search_official(chat_id, &parsed.args[0]).await,
        },
        Err(command::Error::Unknown(name)) => {
            help::reply(chat_id, &command::unknown_text(&name)).await;
        }
        Err(command::Error::InvalidArgs { spec, reason }) => {
            help::reply(chat_id, &command::invalid_text(spec, &reason)).await;
        }
    }
    Ok(())
}
