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
    /// 消息类型, text, image, post(混合文本图片等)
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

impl MessageEvent {
    pub fn chat_id(&self) -> &str {
        &self.message.chat_id
    }

    /// 消息内容 JSON 字符串, 需要二次解析. 但是结构比较复杂, 尤其是对于富文本情况
    ///
    /// 富文本可能会生成额外的字段, 例如 {"title": "标题", "content": [[富文本对象], [富文本对象]], [[图片对象], [图片对象]]}
    ///
    /// 富文本对象形如: {"style":["bold"],"tag":"text","text":"文本"}
    /// 图片对象形如: {"height":int,"width":int,"tag":"img","image_key":""}
    ///
    /// 不过对于单一类型的消息还是容易解析的, 通常直接就是富文本对象/图片对象等
    pub fn raw_content(&self) -> &str {
        &self.message.content
    }

    /// 这里先只处理最基本的文本消息, 并且抛弃了文本中的 @ 标记, 这些在其他字段里还有.
    /// TODO: 以后可以考虑提取富文本中的纯文本
    pub fn text(&self) -> Option<String> {
        if self.message.message_type == "text" {
            // 直接解析文本消息
            let content: serde_json::Value = serde_json::from_str(&self.message.content).ok()?;
            let text = content["text"].as_str()?;
            if text.is_empty() {
                None
            } else {
                // 如果以 @ 开头, 则去掉这一部分
                // TODO: 这只是个临时方案因为这应该只是个标记也可以在文本内部出现
                if text.starts_with("@"){
                    let parts: Vec<&str> = text.splitn(2, char::is_whitespace).collect();
                    if parts.len() == 2 {
                        Some(parts[1].to_string())
                    } else {
                        None
                    }
                } else {
                    Some(text.to_string())
                }
            }
        } else {
            None
        }
    }
}
