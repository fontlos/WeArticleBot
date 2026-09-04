mod add;
mod help;
mod info;
mod init;
mod login;
mod query;
mod search;
mod summary;
mod sync;

use bytes::Bytes;
use lark::event::{EventDispatcher, EventEnvelope, MessageEvent};
use log::{debug, error, warn};

use std::sync::OnceLock;

use crate::command::{self, Query, QuerySub};

// 处理飞书事件
pub async fn handle(event: Bytes) {
    debug!("Received event: {}", String::from_utf8_lossy(&event));

    if let Err(e) = dispatcher().dispatch(&event).await {
        error!("Event dispatch failed: {}", e);
    }
}

/// 事件分发器
static DISPATCHER: OnceLock<EventDispatcher> = OnceLock::new();
fn dispatcher() -> &'static EventDispatcher {
    DISPATCHER.get_or_init(|| {
        let mut dispatcher = EventDispatcher::new();
        dispatcher.fallback(handle_unsupported_event);
        dispatcher.on("im.message.receive_v1", handle_message_event);
        dispatcher
    })
}

/// 处理不支持的事件类型
async fn handle_unsupported_event(envelope: EventEnvelope) -> lark::error::Result<()> {
    warn!("Unsupported event type: {}", envelope.event_type());
    Ok(())
}

/// 处理消息事件
async fn handle_message_event(envelope: EventEnvelope) -> lark::error::Result<()> {
    let msg_event = envelope.parse_event::<MessageEvent>()?;
    let chat_id = msg_event.chat_id();
    let text = msg_event.text().unwrap_or_default();

    match command::parse_cli(&text) {
        Ok(cli) => match cli.command {
            command::Commands::Login => login::scan_login(chat_id).await,
            command::Commands::Info => info::fetch_profile(chat_id).await,
            command::Commands::Init => init::init_bitable(&msg_event).await,
            command::Commands::Search { keyword } => {
                search::search_official(chat_id, &keyword).await
            }
            command::Commands::Add { index } => add::add_account(chat_id, index as usize).await,
            command::Commands::Sync => sync::sync_articles(chat_id).await,
            command::Commands::Summary => summary::summarize_latest(chat_id).await,
            command::Commands::List { id } => search::list_articles(chat_id, &id).await,
            command::Commands::Query(Query { command }) => match command {
                QuerySub::UserId => query::query_user_id(&msg_event).await,
            },
        },
        Err(err) => {
            // warn!("Unknown command: {}", name);
            help::reply(chat_id, &err.to_string()).await;
        }
    }
    Ok(())
}
