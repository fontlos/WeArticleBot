use serde::Deserialize;

use crate::Session;

use crate::api::Res;
use crate::api::docs::drive::folder::FolderMeta;

use super::super::Bitable;

#[derive(Debug, Deserialize)]
struct BitableApp {
    app: BitableMeta,
}

#[derive(Debug, Deserialize)]
pub struct BitableMeta {
    pub app_token: String,
    pub default_table_id: String,
    pub folder_token: String,
    pub name: String,
    pub url: String,
}

impl Session<Bitable> {
    pub async fn create_bitable(
        &self,
        name: &str,
        folder: &FolderMeta,
    ) -> crate::Result<BitableMeta> {
        let url = "https://open.feishu.cn/open-apis/bitable/v1/apps";
        let body = serde_json::json!({
            "name": name,
            "folder_token": &folder.token,
            "time_zone": "Asia/Macau",
        });
        let req = self.client.post(url).json(&body);
        let bytes = self.core().request(req).await?;
        let res: BitableApp = Res::parse(&bytes)?;
        Ok(res.app)
    }
}
