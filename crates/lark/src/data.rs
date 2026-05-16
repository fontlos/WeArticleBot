use serde::Deserialize;

// =====================
// 飞书 API 解析
// =====================

// 统一推送头部
#[derive(Debug, Deserialize)]
pub struct EventEnvelope {
    pub schema: String,
    pub header: EventHeader,
    pub event: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct EventHeader {
    pub event_id: String,
    pub event_type: String,
    pub create_time: String,
    pub tenant_key: String,
    pub app_id: String,
    pub token: String,
}

// 事件: 收到消息
#[derive(Debug, Deserialize)]
pub struct MessageEvent {
    pub message: MessageContent,
    pub sender: Sender,
}

#[derive(Debug, Deserialize)]
pub struct MessageContent {
    pub chat_id: String,
    pub chat_type: String,
    pub content: String, // 这是 JSON 字符串，需要二次解析
    pub message_id: String,
    pub message_type: String,
    pub create_time: String,
    pub update_time: String,
    pub mentions: Vec<Mention>,
}

#[derive(Debug, Deserialize)]
pub struct Mention {
    pub key: String,
    pub id: MentionId,
    pub name: String,
    pub tenant_key: String,
    pub mentioned_type: String,
}

#[derive(Debug, Deserialize)]
pub struct MentionId {
    pub open_id: String,
    pub union_id: String,
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Sender {
    pub sender_id: SenderId,
    pub sender_type: String,
    pub tenant_key: String,
}

#[derive(Debug, Deserialize)]
pub struct SenderId {
    pub open_id: String,
    pub union_id: String,
    pub user_id: Option<String>,
}
