use serde::Deserialize;

// 事件: 收到消息
#[derive(Debug, Deserialize)]
pub struct MessageEvent {
    pub message: MessageContent,
    pub sender: Sender,
}

#[derive(Debug, Deserialize)]
pub struct MessageContent {
    /// 消息所属的聊天 ID, 群聊单聊混用
    pub chat_id: String,
    /// 消息所属的聊天类型, group 或 p2p
    pub chat_type: String,
    /// 消息 ID
    pub message_id: String,
    /// 消息类型, text, image
    pub message_type: String,
    // 这是 JSON 字符串，需要二次解析
    // 根据 message type, 字段可能是 text或image_key, 群聊中字段内容可能有 @_user_1 前缀等
    pub content: String,
    // 消息中提到的用户列表
    pub mentions: Option<Vec<Mention>>,
    pub create_time: String,
    pub update_time: String,
}

#[derive(Debug, Deserialize)]
pub struct Mention {
    /// @_user_{index}, 例如 @_user_1, 标记顺序
    pub key: String,
    pub id: MentionId,
    pub name: String,
    pub tenant_key: String,
    /// user / bot
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
    /// 消息发送者 ID
    pub sender_id: SenderId,
    /// user / bot, 也许还有 app
    pub sender_type: String,
    pub tenant_key: String,
}

#[derive(Debug, Deserialize)]
pub struct SenderId {
    pub open_id: String,
    pub union_id: String,
    pub user_id: Option<String>,
}
