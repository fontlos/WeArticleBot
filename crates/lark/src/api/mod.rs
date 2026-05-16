mod auth;

use crate::error::Result;
use crate::session::Session;

impl Session {
    pub async fn send_text_message(&self, receive_id: &str, receive_id_type: &str, text: &str) -> Result<()> {
        self.refresh_access_token().await?;
        let url = "https://open.feishu.cn/open-apis/im/v1/messages";
        let query = [("receive_id_type", receive_id_type)];
        let content = serde_json::json!({ "text": text }).to_string();
        let json = serde_json::json!({
            "receive_id": receive_id,
            "content": content,
            "msg_type": "text"
        });

        let res = self
            .client
            .post(url)
            .bearer_auth(self.token.load().as_str())
            .query(&query)
            .json(&json)
            .send()
            .await?;

        let bytes = res.bytes().await?;
        println!("Send message response: {}", String::from_utf8_lossy(&bytes));
        Ok(())
    }

    pub async fn reply_to_chat(&self, chat_id: &str, text: &str) -> Result<()> {
        self.send_text_message(chat_id, "chat_id", text).await
    }

    pub async fn reply_to_user(&self, open_id: &str, text: &str) -> Result<()> {
        self.send_text_message(open_id, "open_id", text).await
    }
}
