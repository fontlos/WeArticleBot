use crate::error::Result;
use crate::session::Session;

use super::response::Response;

impl Session {
    pub async fn send_message(
        &self,
        receive_id: &str,
        receive_id_type: &str,
        msg_type: &str,
        content: &str,
    ) -> Result<()> {
        let url = "https://open.feishu.cn/open-apis/im/v1/messages";
        let query = [("receive_id_type", receive_id_type)];
        let json = serde_json::json!({
            "receive_id": receive_id,
            "content": content,
            "msg_type": msg_type
        });

        let req = self.client.post(url).query(&query).json(&json);

        let bytes = self.request(req).await?;

        Response::check(&bytes)?;

        println!("Send message response: {}", String::from_utf8_lossy(&bytes));
        Ok(())
    }

    pub async fn send_text_to_chat(&self, chat_id: &str, text: &str) -> Result<()> {
        let content = serde_json::json!({ "text": text }).to_string();
        self.send_message(chat_id, "chat_id", "text", &content)
            .await
    }

    pub async fn send_text_to_user(&self, open_id: &str, text: &str) -> Result<()> {
        let content = serde_json::json!({ "text": text }).to_string();
        self.send_message(open_id, "open_id", "text", &content)
            .await
    }

    pub async fn send_image_to_chat(&self, chat_id: &str, image_key: &str) -> Result<()> {
        let content = serde_json::json!({ "image_key": image_key }).to_string();
        self.send_message(chat_id, "chat_id", "image", &content)
            .await
    }
}
