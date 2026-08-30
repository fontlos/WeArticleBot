use serde::Deserialize;

use crate::Session;

use crate::api::Res;
use crate::api::docs::drive::folder::Folder;
use crate::error::Error;

use super::super::Bitable;

/// Bitable 应用元信息包装
#[derive(Debug, Deserialize)]
struct BitableApp {
    app: BitableMeta,
}

/// Bitable 应用元信息
#[derive(Debug, Deserialize)]
pub struct BitableMeta {
    pub app_token: String,
    pub default_table_id: String,
    pub folder_token: String,
    pub name: String,
    pub url: String,
}

impl Session<Bitable> {
    /// 创建 Bitable 应用
    pub async fn create_bitable<F>(&self, name: &str, folder: &F) -> crate::Result<BitableMeta>
    where
        F: Folder,
    {
        if !folder.is_folder() {
            return Err(Error::Custom("`folder` must be a folder".to_string()));
        }
        let url = "https://open.feishu.cn/open-apis/bitable/v1/apps";
        let body = serde_json::json!({
            "name": name,
            "folder_token": &folder.token(),
            "time_zone": "Asia/Macau",
        });
        let req = self.client.post(url).json(&body);
        let bytes = self.request(req).await?;
        let res: BitableApp = Res::parse(&bytes)?;
        Ok(res.app)
    }
}
