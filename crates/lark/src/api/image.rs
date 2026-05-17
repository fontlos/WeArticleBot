use reqwest::multipart::{Form, Part};
use serde::Deserialize;

use crate::error::Result;
use crate::session::Session;

use super::response::Response;

#[derive(Debug, Deserialize)]
struct UploadImageResponse {
    image_key: String,
}

impl Session {
    pub async fn upload_image(&self, img: &[u8]) -> Result<String> {
        let url = "https://open.feishu.cn/open-apis/im/v1/images";
        let form = Form::new().text("image_type", "message").part(
            "image",
            Part::bytes(img.to_vec())
                .file_name("image.jpg")
                .mime_str("image/jpeg")?,
        );

        let req = self.client.post(url).multipart(form);

        let bytes = self.request(req).await?;
        println!("Upload image response: {}", String::from_utf8_lossy(&bytes));

        let res: UploadImageResponse = Response::parse(&bytes)?;
        Ok(res.image_key)
    }
}
