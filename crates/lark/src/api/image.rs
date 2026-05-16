use reqwest::multipart::{Form, Part};

use crate::error::Result;
use crate::session::Session;

impl Session {
    pub async fn upload_img(&self, img: &[u8]) -> Result<String> {
        self.refresh_access_token().await?;
        let url = "https://open.feishu.cn/open-apis/im/v1/images";
        let form = Form::new()
            .text("image_type", "message")
            .part("image", Part::bytes(img.to_vec())
                .file_name("image.jpg")
                .mime_str("image/jpeg")?);

        let res = self.client
            .post(url)
            .bearer_auth(self.token.load().as_str())
            .multipart(form)
            .send()
            .await?;

        let bytes = res.bytes().await?;
        println!("Upload image response: {}", String::from_utf8_lossy(&bytes));

        let json: serde_json::Value = serde_json::from_slice(&bytes)?;
        let image_key = json["data"]["image_key"].as_str().unwrap_or("");
        Ok(image_key.to_string())
    }
}
