use crate::error::Result;
use crate::session::Session;

use super::response::Response;

#[derive(Clone, Debug, Default)]
pub struct Message {
    /// 消息接收者 ID, 可以是 chat_id / open_id / user_id
    receive_id: String,
    /// 消息接收者 ID 类型, 可以是 "chat_id" / "open_id" / "user_id"
    receive_id_type: &'static str,
    /// 消息类型, 可以是 "text" / "image" / "file"
    msg_type: &'static str,
    /// 消息内容, 根据消息类型不同而不同, 例如 text 消息是 {"text": "hello"}, image 消息是 {"image_key": "img_xxx"}
    content: String,
}

impl Message {
    pub fn to_chat(id: &str) -> Self {
        Self {
            receive_id: id.to_string(),
            receive_id_type: "chat_id",
            ..Default::default()
        }
    }

    pub fn to_user(id: &str) -> Self {
        Self {
            receive_id: id.to_string(),
            receive_id_type: "open_id",
            ..Default::default()
        }
    }

    pub fn text(mut self, text: &str) -> Self {
        self.msg_type = "text";
        self.content = serde_json::json!({ "text": text }).to_string();
        self
    }

    pub fn image(mut self, image_key: &str) -> Self {
        self.msg_type = "image";
        self.content = serde_json::json!({ "image_key": image_key }).to_string();
        self
    }
}

impl Session {
    pub async fn send_message(&self, msg: Message) -> Result<()> {
        let url = "https://open.feishu.cn/open-apis/im/v1/messages";
        let query = [("receive_id_type", msg.receive_id_type)];
        let json = serde_json::json!({
            "receive_id": msg.receive_id,
            "content": msg.content,
            "msg_type": msg.msg_type
        });

        let req = self.client.post(url).query(&query).json(&json);

        let bytes = self.request(req).await?;

        Response::check(&bytes)?;

        println!("Send message response: {}", String::from_utf8_lossy(&bytes));
        Ok(())
    }
}
